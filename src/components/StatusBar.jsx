import React, { useRef } from 'react';
import { Box, Text, useStdout } from 'ink';
import { useOnClick } from '@ink-tools/ink-mouse';

export default function StatusBar({ children, rightContent }) {
  const { stdout } = useStdout();
  const width = Math.max((stdout?.columns || 80) - 5, 60);

  return (
    <Box marginTop={1} flexDirection="column">
      <Box>
        <Text color="gray">{'─'.repeat(width)}</Text>
      </Box>
      <Box justifyContent="space-between">
        <Box>
          {children}
        </Box>
        {rightContent && <Box>{rightContent}</Box>}
      </Box>
    </Box>
  );
}

export function KeyHint({ keyName, description, onClick }) {
  const ref = useRef(null);
  useOnClick(ref, onClick || (() => {}));

  return (
    <Box ref={ref} marginRight={2}>
      <Text color="yellow">({keyName})</Text>
      <Text color="gray">{description}</Text>
    </Box>
  );
}
