using System;
using System.Collections.Concurrent;
using System.IO;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using UnityEngine;

namespace SecuritySandbox.Networking
{
    public sealed class RustTcpClient : MonoBehaviour
    {
        [SerializeField] private RustServerSettings settings;

        private TcpClient client;
        private StreamReader reader;
        private StreamWriter writer;
        private Thread readerThread;
        private volatile bool running;

        private readonly ConcurrentQueue<string> incomingLines = new();

        public bool IsConnected => client != null && client.Connected;

        public event Action<string> LineReceived;

        private void Start()
        {
            Connect();
        }

        private void Update()
        {
            while (incomingLines.TryDequeue(out string line))
            {
                LineReceived?.Invoke(line);
            }
        }

        private void OnDestroy()
        {
            Disconnect();
        }

        public void Connect()
        {
            if (settings == null)
            {
                Debug.LogError("RustTcpClient is missing RustServerSettings.");
                return;
            }

            try
            {
                client = new TcpClient();
                client.Connect(settings.Host, settings.Port);

                NetworkStream stream = client.GetStream();

                reader = new StreamReader(stream, Encoding.UTF8);
                writer = new StreamWriter(stream, new UTF8Encoding(false))
                {
                    AutoFlush = true,
                    NewLine = "\n"
                };

                running = true;
                readerThread = new Thread(ReadLoop)
                {
                    IsBackground = true,
                    Name = "Rust TCP Reader"
                };
                readerThread.Start();

                SendLine(RustJsonMessages.Join(settings.PlayerId));

                Debug.Log($"Connected to Rust server at {settings.Host}:{settings.Port}");
            }
            catch (Exception exception)
            {
                Debug.LogError($"Failed to connect to Rust server: {exception.Message}");
                Disconnect();
            }
        }

        public void SendLine(string line)
        {
            if (writer == null)
            {
                return;
            }

            try
            {
                writer.WriteLine(line);
            }
            catch (Exception exception)
            {
                Debug.LogError($"Failed to send line to Rust server: {exception.Message}");
            }
        }

        private void ReadLoop()
        {
            while (running)
            {
                try
                {
                    string line = reader.ReadLine();

                    if (line == null)
                    {
                        break;
                    }

                    incomingLines.Enqueue(line);
                }
                catch (IOException)
                {
                    break;
                }
                catch (ObjectDisposedException)
                {
                    break;
                }
                catch (Exception exception)
                {
                    incomingLines.Enqueue("{\"type\":\"Rejected\",\"data\":{\"reason\":\"reader error: " + Escape(exception.Message) + "\"}}");
                    break;
                }
            }
        }

        private void Disconnect()
        {
            running = false;

            try
            {
                client?.Close();
            }
            catch
            {
                // ignored during shutdown
            }

            client = null;
            reader = null;
            writer = null;
        }

        private static string Escape(string value)
        {
            return value.Replace("\\", "\\\\").Replace("\"", "\\\"");
        }
    }
}