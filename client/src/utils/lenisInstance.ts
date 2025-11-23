import Lenis from 'lenis';
import { frame } from 'motion';

export const lenisInstance = new Lenis({
  syncTouch: true,
});

export function activateLenis() {
  function update(data: { timestamp: number }) {
    lenisInstance.raf(data.timestamp);
  }

  frame.update(update, true);
}
