using System.Globalization;

namespace SecuritySandbox.Networking
{
    public static class RustJsonMessages
    {
        public static string Join(ulong playerId)
        {
            return "{\"type\":\"Join\",\"data\":{\"player_id\":" + playerId + "}}";
        }

        public static string Ping(ulong clientTimeMs)
        {
            return "{\"type\":\"Ping\",\"data\":{\"client_time_ms\":" + clientTimeMs + "}}";
        }

        public static string Input(
            ulong playerId,
            ulong sequence,
            ulong clientTimeMs,
            NetworkVec2 movement,
            bool fire,
            NetworkVec2 claimedPosition
        )
        {
            return
                "{\"type\":\"Input\",\"data\":{"
                + "\"player_id\":" + playerId + ","
                + "\"sequence\":" + sequence + ","
                + "\"client_time_ms\":" + clientTimeMs + ","
                + "\"movement\":{\"x\":" + Float(movement.x) + ",\"y\":" + Float(movement.y) + "},"
                + "\"fire\":" + Bool(fire) + ","
                + "\"claimed_position\":{\"x\":" + Float(claimedPosition.x) + ",\"y\":" + Float(claimedPosition.y) + "}"
                + "}}";
        }

        private static string Float(float value)
        {
            return value.ToString("0.######", CultureInfo.InvariantCulture);
        }

        private static string Bool(bool value)
        {
            return value ? "true" : "false";
        }
    }
}