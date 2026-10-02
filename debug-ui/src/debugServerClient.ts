import type { PlayableScore } from './play';

export type ServerEvent =
  | { type: 'progress'; progress: number; furthest: number; total: number; iteration: number }
  | { type: 'ok'; mxl: string; playable: PlayableScore }
  | { type: 'no-solution' };

export type ConnectionStatus = 'connecting' | 'open' | 'standby';

export interface DebugServerClient {
  close(): void;
}

const RETRY_MS = 1000;

export function subscribe(
  onEvent: (event: ServerEvent) => void,
  onStatus?: (status: ConnectionStatus) => void,
): DebugServerClient {
  let source: EventSource | null = null;
  let retryTimer: ReturnType<typeof setTimeout> | null = null;
  let closed = false;

  const connect = () => {
    if (closed) return;

    source = new EventSource('/events');

    source.onopen = () => onStatus?.('open');

    source.onerror = () => {
      source?.close();
      source = null;
      onStatus?.('standby');
      if (!closed) {
        retryTimer = setTimeout(connect, RETRY_MS);
      }
    };

    source.onmessage = (ev: MessageEvent<string>) => {
      onEvent(JSON.parse(ev.data) as ServerEvent);
    };
  };

  onStatus?.('connecting');
  connect();

  return {
    close() {
      closed = true;
      if (retryTimer !== null) {
        clearTimeout(retryTimer);
        retryTimer = null;
      }
      source?.close();
      source = null;
    },
  };
}
