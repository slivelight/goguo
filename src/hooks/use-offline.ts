import { useState, useEffect } from 'react';
import { useServiceStore } from '../stores/service-store';

export function useOffline(): { isOffline: boolean; lastKnownStatus: boolean } {
  const { mihomoRunning, fetchServiceStatus } = useServiceStore();
  const [isOffline, setIsOffline] = useState(false);
  const [lastKnownStatus, setLastKnownStatus] = useState(mihomoRunning);

  useEffect(() => {
    const checkConnection = async () => {
      try {
        await fetchServiceStatus();
        setIsOffline(false);
        setLastKnownStatus(mihomoRunning);
      } catch {
        setIsOffline(true);
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 5000);

    return () => clearInterval(interval);
  }, [fetchServiceStatus, mihomoRunning]);

  return { isOffline, lastKnownStatus };
}