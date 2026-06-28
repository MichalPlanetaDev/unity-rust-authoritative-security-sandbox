using UnityEngine;

namespace SecuritySandbox.Networking
{
    public sealed class SuspiciousInputTester : MonoBehaviour
    {
        [SerializeField] private RustTcpClient tcpClient;
        [SerializeField] private RustServerSettings settings;

        private ulong sequence;

        private void Update()
        {
            if (tcpClient == null || settings == null || !tcpClient.IsConnected)
            {
                return;
            }

            if (Input.GetKeyDown(KeyCode.F1))
            {
                SendImpossibleMovement();
            }

            if (Input.GetKeyDown(KeyCode.F2))
            {
                SendRepeatedFire();
            }

            if (Input.GetKeyDown(KeyCode.F3))
            {
                SendRepeatedSequence();
            }
        }

        private void SendImpossibleMovement()
        {
            sequence++;

            string json = RustJsonMessages.Input(
                settings.PlayerId,
                sequence,
                ClientTimeMs(),
                new NetworkVec2(1.0f, 0.0f),
                false,
                new NetworkVec2(100.0f, 0.0f)
            );

            tcpClient.SendLine(json);
            Debug.Log("Sent suspicious impossible movement input.");
        }

        private void SendRepeatedFire()
        {
            sequence++;

            string first = RustJsonMessages.Input(
                settings.PlayerId,
                sequence,
                ClientTimeMs(),
                new NetworkVec2(0.0f, 0.0f),
                true,
                new NetworkVec2(transform.position.x, transform.position.z)
            );

            sequence++;

            string second = RustJsonMessages.Input(
                settings.PlayerId,
                sequence,
                ClientTimeMs(),
                new NetworkVec2(0.0f, 0.0f),
                true,
                new NetworkVec2(transform.position.x, transform.position.z)
            );

            tcpClient.SendLine(first);
            tcpClient.SendLine(second);
            Debug.Log("Sent suspicious repeated fire input.");
        }

        private void SendRepeatedSequence()
        {
            ulong repeated = sequence;

            string json = RustJsonMessages.Input(
                settings.PlayerId,
                repeated,
                ClientTimeMs(),
                new NetworkVec2(1.0f, 0.0f),
                false,
                new NetworkVec2(transform.position.x, transform.position.z)
            );

            tcpClient.SendLine(json);
            Debug.Log("Sent suspicious repeated sequence input.");
        }

        private ulong ClientTimeMs()
        {
            return (ulong)(Time.realtimeSinceStartup * 1000.0f);
        }
    }
}