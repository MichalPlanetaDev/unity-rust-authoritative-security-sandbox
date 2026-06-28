using System;

namespace SecuritySandbox.Networking
{
    [Serializable]
    public struct NetworkVec2
    {
        public float x;
        public float y;

        public NetworkVec2(float x, float y)
        {
            this.x = x;
            this.y = y;
        }
    }

    [Serializable]
    public sealed class ServerEnvelope
    {
        public string type;
        public ServerData data;
    }

    [Serializable]
    public sealed class ServerData
    {
        public ulong player_id;
        public NetworkVec2 position;
        public int health;
        public bool alive;
        public ulong server_time_ms;
        public ulong client_time_ms;
        public string reason;
    }
}