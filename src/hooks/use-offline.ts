import { useState, useEffect, useRef } from 'react';
import { useServiceStore } from '../stores/service-store';

export function useOffline(): { isOffline: boolean; lastKnownStatus: boolean } {
  const fetchServiceStatus = useServiceStore(state => state.fetchServiceStatus);
  const mihomoRunning = useServiceStore(state => state.mihomoRunning);
  const [isOffline, setIsOffline] = useState(false);
  const [lastKnownStatus, setLastKnownStatus] = useState(mihomoRunning);

  // Read latest value via ref — must NOT be in the useEffect deps array,
  // otherwise every mihomoRunning change re-triggers the effect and the
  // immediate checkConnection() IPC call, creating a tight loop that
  // freezes the WebKitGTK main thread.
  const mihomoRunningRef = useRef(mihomoRunning);
  mihomoRunningRef.current = mihomoRunning;

  useEffect(() => {
    const checkConnection = async () => {
      try {
        await fetchServiceStatus();
        setIsOffline(false);
        setLastKnownStatus(mihomoRunningRef.current);
      } catch {
        setIsOffline(true);
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 5000);

    return () => clearInterval(interval);
  }, [fetchServiceStatus]);

  return { isOffline, lastKnownStatus };
}