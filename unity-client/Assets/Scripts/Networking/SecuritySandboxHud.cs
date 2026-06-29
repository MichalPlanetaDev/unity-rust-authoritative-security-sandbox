using UnityEngine;

namespace SecuritySandbox.Networking
{
    public sealed class SecuritySandboxHud : MonoBehaviour
    {
        [SerializeField] private RustTcpClient tcpClient;
        [SerializeField] private Transform player;

        private string lastServerLine = "none";
        private string connectionState = "disconnected";

        private void OnEnable()
        {
            if (tcpClient != null)
            {
                tcpClient.LineReceived += HandleServerLine;
            }
        }

        private void OnDisable()
        {
            if (tcpClient != null)
            {
                tcpClient.LineReceived -= HandleServerLine;
            }
        }

        private void Update()
        {
            connectionState = tcpClient != null && tcpClient.IsConnected
                ? "connected"
                : "disconnected";
        }

        private void OnGUI()
        {
            const int width = 760;
            const int height = 190;

            GUILayout.BeginArea(new Rect(16, 16, width, height), GUI.skin.box);

            GUILayout.Label("unity-rust-authoritative-security-sandbox");
            GUILayout.Space(6);

            GUILayout.Label($"Connection: {connectionState}");

            if (player != null)
            {
                Vector3 position = player.position;
                GUILayout.Label($"Local player position: x={position.x:0.00}, y={position.y:0.00}, z={position.z:0.00}");
            }

            GUILayout.Space(6);
            GUILayout.Label("Controls:");
            GUILayout.Label("WASD / Arrows = movement | Space = fire | F1 = impossible movement | F2 = repeated fire | F3 = repeated sequence");

            GUILayout.Space(6);
            GUILayout.Label("Last server line:");
            GUILayout.TextArea(lastServerLine, GUILayout.Height(48));

            GUILayout.EndArea();
        }

        private void HandleServerLine(string line)
        {
            lastServerLine = line;
        }
    }
}