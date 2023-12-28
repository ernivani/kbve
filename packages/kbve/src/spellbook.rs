//!         [SPELLBOOK]
//?         Collection of all the v2 Macros

#[macro_export]
macro_rules! spellbook_create_jwt {
	($uuid:expr, $email:expr, $username:expr, $secret:expr, $hours:expr) => {
		{

		use jsonwebtoken::{encode, EncodingKey, Header};

        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::minutes($hours * 60);

        let jwt_token = encode(
            &Header::default(),
            &crate::runes::TokenRune {
                uuid: $uuid.to_string(),
                email: $email.to_string(),
                username: $username.to_string(),
                iat: now.timestamp() as usize,
                exp: exp.timestamp() as usize,
            },
            &EncodingKey::from_secret($secret.as_bytes()),
        ).unwrap(); 

		jwt_token
		}
	};
}

#[macro_export]
macro_rules! spellbook_create_cookie {
	($name:expr, $token:expr, $duration:expr) => {
		axum_extra::extract::cookie::Cookie::build($name, $token)
			.path("/")
			.max_age(time::Duration::hours($duration))
			.same_site(axum_extra::extract::cookie::SameSite::Lax)
			.http_only(true)
			.finish()
	};
}

#[macro_export]
macro_rules! spellbook_get_global {
	($key:expr, $err:expr) => {
        match crate::runes::GLOBAL.get() {
            Some(global_map) => match global_map.get($key) {
                Some(value) => Ok(value.value().clone()), // Assuming you want to clone the value
                None => Err($err),
            },
            None => Err("invalid_global_map"),
        }
	};
}

// #[macro_export]
// macro_rules! spellbook_get_pool {
// 	($pool:expr) => {
//         match $pool.get() {
//             Ok(conn) => conn,
//             Err(_) => return Err("pool_fail"),
//         }
// 	};
// }

/** 
	This macro is a utility for working with database connection pools.
	It tries to retrieve a connection from the provided pool.
	If successful, the connection is returned for further use.
	If there's an error in obtaining a connection, it handles the error by immediately returning an HTTP response with an appropriate error message and status code. This macro ensures a uniform way of handling database pool errors across different parts of an Axum application.
**/

// The `spellbook_pool` macro is defined using Rust's macro_rules! system.
// This macro is designed to simplify the process of obtaining a database connection from a connection pool.
#[macro_export]
macro_rules! spellbook_pool {
	// The macro takes a single argument, `$pool`, which represents the connection pool.
	($pool:expr) => {
        // Attempt to get a database connection from the pool.
        match $pool.get() {
            // If successful, the obtained connection (`conn`) is returned for use.
            Ok(conn) => conn,

            // If there's an error (e.g., the pool is exhausted or connection failed),
            // the macro returns an HTTP response indicating an internal server error.
            // This return statement is designed to exit from the calling function.
            Err(_) => return (
                // Sets the HTTP status code to UNAUTHORIZED (401).
                // Although the error is from the database, the response indicates a more general server error.
                axum::http::StatusCode::UNAUTHORIZED,

                // The body of the response is a JSON object with an error message.
                axum::Json(serde_json::json!({"error": "db_error"})),

                // Converts the tuple into an Axum response type.
            ).into_response(),
        }
	};
}


#[macro_export]
macro_rules! spellbook_complete {
	($spell:expr) => {
        return (axum::http::StatusCode::OK, axum::Json(serde_json::json!({"data": $spell}))).into_response()
	};
}

#[macro_export]
macro_rules! spellbook_username {
	($username:expr) => {
        match crate::utility::sanitize_username($username) {
            Ok(username) => username,
            Err(e) => return (axum::http::StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({"error": format!("{}",e)}))).into_response()
        }
	};
}

#[macro_export]
macro_rules! spellbook_ulid {
	($ulid:expr) => {
        match crate::utility::sanitize_ulid($ulid) {
            Ok(ulid) => ulid,
            Err(e) => return (axum::http::StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({"error": format!("{}",e)}))).into_response()
        }
	};
}

#[macro_export]
macro_rules! spellbook_email {
	($email:expr) => {
        match crate::utility::sanitize_email($email) {
            Ok(email) => email,
            Err(e) => return (axum::http::StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({"error": format!("{}",e)}))).into_response()
        }
	};
}

/**

In the spellbook_sanitize_fields macro:
	- It takes any struct ($struct) and a list of fields within that struct.
	- For each field, if it is an Option<String> and currently has a value (Some), that value is sanitized using the crate::harden::sanitize_string_limit function.
	- The macro is designed to be reusable for any struct with fields that need sanitizing and can handle multiple fields at once.
	- This macro simplifies the process of sanitizing multiple fields in a struct, ensuring that each specified field is sanitized if it contains a value. 
	It reduces code repetition and improves readability by abstracting the common pattern of sanitizing multiple optional fields.
**/

// This is a macro definition using Rust's macro_rules! system.
// It is designed to generalize the process of sanitizing fields in a struct.
#[macro_export]
macro_rules! spellbook_sanitize_fields {
	// The macro takes two types of input:
	// 1. $struct:expr, which represents the struct instance whose fields need sanitizing.
	// 2. $($field:ident),+, which is a variadic list of field identifiers that need to be sanitized.
	($struct:expr, $($field:ident),+) => {
        $(
            // This loop iterates over each field specified in the macro invocation.
            if let Some(ref mut value) = $struct.$field {
                // If the field ($field) is Some (i.e., it's not None), then the field's value is sanitized.
                // `*value` dereferences the Option to get a mutable reference to the contained String.
                // `crate::harden::sanitize_string_limit` is called to sanitize the value.
                // This could include operations like trimming, removing special characters, etc.
                *value = crate::harden::sanitize_string_limit(value);
            }
        )+
	};
}