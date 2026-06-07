import { useEffect, useRef, useState } from "react";
import { WS_URL } from "../api/client";
import type { ServerMessage } from "../types";

export type ConnectionStatus = "connecting" | "connected" | "disconnected";

export function useTelemetrySocket(
  onMessage: (message: ServerMessage) => void,
) {
  const [status, setStatus] = useState<ConnectionStatus>("connecting");
  const callbackRef = useRef(onMessage);

  useEffect(() => {
    callbackRef.current = onMessage;
  }, [onMessage]);

  useEffect(() => {
    let socket: WebSocket | null = null;
    let retryTimer: number | undefined;
    let closedByCleanup = false;

    const connect = () => {
      setStatus("connecting");
      socket = new WebSocket(WS_URL);

      socket.onopen = () => setStatus("connected");
      socket.onmessage = (event) => {
        try {
          callbackRef.current(JSON.parse(event.data) as ServerMessage);
        } catch {
          console.warn("Received malformed WebSocket message");
        }
      };
      socket.onerror = () => socket?.close();
      socket.onclose = () => {
        setStatus("disconnected");
        if (!closedByCleanup) {
          retryTimer = window.setTimeout(connect, 1800);
        }
      };
    };

    connect();

    return () => {
      closedByCleanup = true;
      window.clearTimeout(retryTimer);
      socket?.close();
    };
  }, []);

  return status;
}

