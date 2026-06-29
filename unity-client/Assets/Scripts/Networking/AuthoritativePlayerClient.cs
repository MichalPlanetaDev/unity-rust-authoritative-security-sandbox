using UnityEngine;

namespace SecuritySandbox.Networking
{
    public sealed class AuthoritativePlayerClient : MonoBehaviour
    {
        [SerializeField] private RustTcpClient tcpClient;
        [SerializeField] private RustServerSettings settings;
        [SerializeField] private float localMoveSpeed = 5.0f;

        private ulong sequence;
        private float startedAt;
        private Vector3 serverPosition;

        private void Awake()
        {
            startedAt = Time.realtimeSinceStartup;
            serverPosition = transform.position;
        }

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
            if (tcpClient == null || settings == null || !tcpClient.IsConnected)
            {
                return;
            }

            Vector2 movement = ReadMovementInput();
            bool fire = Input.GetKeyDown(KeyCode.Space);

            if (movement.sqrMagnitude <= 0.0001f && !fire)
            {
                return;
            }

            sequence++;

            Vector3 predictedPosition = transform.position
                + new Vector3(movement.x, 0.0f, movement.y) * localMoveSpeed * Time.deltaTime;

            transform.position = predictedPosition;

            NetworkVec2 networkMovement = new NetworkVec2(movement.x, movement.y);
            NetworkVec2 claimedPosition = new NetworkVec2(predictedPosition.x, predictedPosition.z);

            string json = RustJsonMessages.Input(
                settings.PlayerId,
                sequence,
                ClientTimeMs(),
                networkMovement,
                fire,
                claimedPosition
            );

            tcpClient.SendLine(json);
        }

        private Vector2 ReadMovementInput()
        {
            Vector2 input = new Vector2(
                Input.GetAxisRaw("Horizontal"),
                Input.GetAxisRaw("Vertical")
            );

            return input.sqrMagnitude > 1.0f ? input.normalized : input;
        }

        private ulong ClientTimeMs()
        {
            return (ulong)((Time.realtimeSinceStartup - startedAt) * 1000.0f);
        }

        private void HandleServerLine(string line)
        {
            ServerEnvelope envelope = JsonUtility.FromJson<ServerEnvelope>(line);

            if (envelope == null)
            {
                Debug.LogWarning($"Invalid server line: {line}");
                return;
            }

            switch (envelope.type)
            {
                case "Welcome":
                    Debug.Log($"Server accepted player {envelope.data.player_id}");
                    break;

                case "Snapshot":
                    ApplySnapshot(envelope.data);
                    break;

                case "Rejected":
                    Debug.LogWarning($"Server rejected input: {envelope.data.reason}");
                    break;

                case "Pong":
                    Debug.Log($"Pong server_time_ms={envelope.data.server_time_ms}");
                    break;

                default:
                    Debug.Log($"Server line: {line}");
                    break;
            }
        }

        private void ApplySnapshot(ServerData data)
        {
            serverPosition = new Vector3(data.position.x, transform.position.y, data.position.y);

            transform.position = Vector3.Lerp(transform.position, serverPosition, 0.35f);
        }
    }
}