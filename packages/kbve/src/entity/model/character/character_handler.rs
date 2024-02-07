use std::sync::{ Arc };

use axum::{
	http::StatusCode,
	extract::{ Extension, Json },
	response::IntoResponse,
};

use diesel::prelude::*;
use diesel::insert_into;

use tokio::task;

use chrono::Utc;

use crate::db::{ Pool };

use crate::schema::{ characters };

use crate::models::{ Character };

use crate::session::{ KbveState, TokenJWT };

use crate::response::{ GenericResponse };

use crate::{ spellbook_pool_conn, spellbook_generate_ulid_bytes };

use crate::utility::{ convert_ulid_string_to_bytes };

use jedi::builder::ValidatorBuilder;

use jsonwebtoken::TokenData;

use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CharacterCreationRequest {
	pub name: String,
	pub description: String,
}

pub async fn hazardous_blocking_boolean_character_name_slot_open(
	dirty_name: String,
	pool: Arc<Pool>
) -> Result<bool, &'static str> {
	let result = task
		::spawn_blocking(move || {
			let mut conn = spellbook_pool_conn!(pool);

			characters::table
				.filter(characters::name.eq(dirty_name))
				.select(characters::cid)
				.first::<Vec<u8>>(&mut conn)
				.map(|_| false)
				.or_else(|err| {
					match err {
						diesel::result::Error::NotFound => Ok(true),
						_ => Err("db_error"),
					}
				})
		}).await
		.expect("spawn_blocking failed");

	result
}

pub async fn hazardous_blocking_character_viewer_from_name(
	character_name: String,
	pool: Arc<Pool>
) -> Result<Character, &'static str> {
	let result = task
		::spawn_blocking(move || {
			let mut conn = spellbook_pool_conn!(pool);

			match
				characters::table
					.filter(characters::name.eq(character_name))
					.first::<Character>(&mut conn)
			{
				Ok(character) => Ok(character),
				Err(diesel::result::Error::NotFound) =>
					Err("Character was not found"),
				Err(_) => Err("Database Error"),
			}
		}).await
		.expect("spawn_blocking failed");

	result
}

pub async fn hazardous_blocking_create_character_from_user(
	dirty_name: String,
	dirty_description: String,
	dirty_user_id: Vec<u8>,
	pool: Arc<Pool>
) -> Result<bool, &'static str> {
	let result = task
		::spawn_blocking(move || {
			let mut conn = spellbook_pool_conn!(pool);

			let clean_cid = spellbook_generate_ulid_bytes!();

			insert_into(characters::table)
				.values((
					characters::id.eq(0),
					characters::cid.eq(clean_cid),
					characters::userid.eq(dirty_user_id),
					characters::hp.eq(100),
					characters::mp.eq(100),
					characters::ep.eq(100),
					characters::health.eq(100),
					characters::mana.eq(100),
					characters::energy.eq(100),
					characters::armour.eq(1),
					characters::agility.eq(1),
					characters::strength.eq(1),
					characters::intelligence.eq(1),
					characters::name.eq(dirty_name),
					characters::description.eq(dirty_description),
					characters::experience.eq(0),
					characters::reputation.eq(0),
					characters::faith.eq(1),
				))
				.execute(&mut conn)
				.map(|_| true)
				.map_err(|_| "Failed to insert character into database")
		}).await
		.expect("spawn_blocking failed");

	result
}

pub async fn character_creation_handler(
	Extension(state): Extension<Arc<KbveState>>,
	Extension(mut privatedata): Extension<TokenData<TokenJWT>>,
	Json(payload): Json<CharacterCreationRequest>
) -> impl IntoResponse {
	let mut conn = match state.db_pool.get() {
		Ok(conn) => conn,
		Err(e) => {
			let error_response = GenericResponse::error(
				json!({}),
				json!("Failed to acquire database connection"),
				e.to_string(),
				StatusCode::INTERNAL_SERVER_ERROR
			);
			return error_response.into_response();
		}
	};

	let user_id = match
		ValidatorBuilder::<String, String>
			::new()
			.clean_or_fail()
			.ulid()
			.validate(privatedata.claims.userid)
	{
		Ok(user_id) => user_id,
		Err(validation_error) => {
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "User ID Validation failed", "details": validation_error}),
				validation_error.join(", "),
				StatusCode::BAD_REQUEST
			);
			return error_response.into_response();
		}
	};

	let name = match
		ValidatorBuilder::<String, String>
			::new()
			.clean_or_fail()
			.username()
			.validate(payload.name)
	{
		Ok(name) => name,
		Err(validation_error) => {
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "Character Name Validation failed", "details": validation_error}),
				validation_error.join(", "),
				StatusCode::BAD_REQUEST
			);
			return error_response.into_response();
		}
	};

	let description = match
		ValidatorBuilder::<String, String>
			::new()
			.clean_or_fail()
			.validate(payload.description)
	{
		Ok(description) => description,
		Err(validation_error) => {
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "Character Description Validation failed", "details": validation_error}),
				validation_error.join(", "),
				StatusCode::BAD_REQUEST
			);
			return error_response.into_response();
		}
	};

	// TODO - Integrate the General Input Regex for the Description.

	let is_slot_open = match
		hazardous_blocking_boolean_character_name_slot_open(
			name.clone(),
			state.db_pool.clone()
		).await
	{
		Ok(true) => {
			// The slot is open, proceed with character creation or other logic
		}
		Ok(false) => {
			// The name is taken, handle accordingly (e.g., return an error response)
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "Character name is already taken"}),
				"Character name is not available".to_string(),
				StatusCode::BAD_REQUEST
			);
			return error_response.into_response();
		}
		Err(_) => {
			// Handle database or other errors
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "Failed to check character name availability"}),
				"Database error or other error occurred".to_string(),
				StatusCode::INTERNAL_SERVER_ERROR
			);
			return error_response.into_response();
		}
	};

	let byte_ulid = match convert_ulid_string_to_bytes(&user_id) {
		Ok(bytes) => bytes,
		Err(e) => {
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "Failed to convert user_id to byte Ulid"}),
				"ULID error or other error occurred".to_string(),
				StatusCode::INTERNAL_SERVER_ERROR
			);
			return error_response.into_response();
		}
	};

	let creation_result = hazardous_blocking_create_character_from_user(
		name.clone(),
		description.clone(),
		byte_ulid,
		state.db_pool.clone()
	).await;

	match creation_result {
		Ok(_) => {}
		Err(error_message) => {
			let error_response = GenericResponse::error(
				json!({}),
				json!({"error": "Failed to create character within the database"}),
				"Character creation has failed!".to_string(),
				StatusCode::INTERNAL_SERVER_ERROR
			);
			return error_response.into_response();
		}
	}

	let success_response = GenericResponse::new(
		json!({"character_id": "some_character_id"}), // Example success data
		json!(
			format!(
				"Character {} created successfully, Name: {}, Description {},",
				user_id,
				name,
				description
			)
		),
		StatusCode::CREATED
	);
	success_response.into_response()
}