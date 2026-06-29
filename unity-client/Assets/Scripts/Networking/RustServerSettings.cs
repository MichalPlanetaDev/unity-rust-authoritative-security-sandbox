using UnityEngine;

namespace SecuritySandbox.Networking
{
    [CreateAssetMenu(
        fileName = "RustServerSettings",
        menuName = "Security Sandbox/Rust Server Settings"
    )]
    public sealed class RustServerSettings : ScriptableObject
    {
        [SerializeField] private string host = "127.0.0.1";
        [SerializeField] private int port = 4000;
        [SerializeField] private ulong playerId = 10;

        public string Host => host;
        public int Port => port;
        public ulong PlayerId => playerId;
    }
}