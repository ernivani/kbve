using KBVE.Services;
using UnityEngine;

public class BootUp : MonoBehaviour
{
  void Awake()
  {
    // Ensure the Services instance is ready or initialized
    var servicesInstance = Services.Instance;

    // Register AuthenticationService
    var authService = gameObject.AddComponent<AuthenticationService>();
    Services.Instance.RegisterService<IAuthenticationService>(authService);

    // Register SceneLoaderService
    var sceneLoaderService = gameObject.AddComponent<SceneLoaderService>();
    Services.Instance.RegisterService<ISceneLoaderService>(sceneLoaderService);
  }
}
