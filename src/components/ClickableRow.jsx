import React, { useRef, useCallback } from 'react';
import { Box } from 'ink';
import { useOnClick } from '@ink-tools/ink-mouse';

const DOUBLE_CLICK_THRESHOLD = 400;

let globalLastClickTime = 0;
let globalLastClickIndex = -1;
let globalClickLock = false;

export default function ClickableRow({ index, onSelect, onActivate, children }) {
  const ref = useRef(null);

  const handleClick = useCallback(() => {
    if (globalClickLock) return;
    globalClickLock = true;
    setTimeout(() => { globalClickLock = false; }, 50);

    const now = Date.now();
    const timeDiff = now - globalLastClickTime;

    if (timeDiff < DOUBLE_CLICK_THRESHOLD && globalLastClickIndex === index) {
      onActivate?.(index);
      globalLastClickTime = 0;
      globalLastClickIndex = -1;
    } else {
      onSelect?.(index);
      globalLastClickTime = now;
      globalLastClickIndex = index;
    }
  }, [index, onSelect, onActivate]);

  useOnClick(ref, handleClick);

  return (
    <Box ref={ref}>
      {children}
    </Box>
  );
}
