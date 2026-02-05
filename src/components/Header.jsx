import React from 'react';
import { Box, Text, useStdout } from 'ink';

export default function Header({ title, subtitle, loading }) {
  const { stdout } = useStdout();
  const width = Math.max((stdout?.columns || 80) - 5, 60);

  const displaySubtitle = loading ? 'Loading...' : subtitle;

  return (
    <Box flexDirection="column" marginBottom={1}>
      <Box>
        <Text bold color="cyan">clipboard</Text>
        {title && (
          <>
            <Text color="gray"> - </Text>
            <Text bold>{title}</Text>
          </>
        )}
        {displaySubtitle && (
          <Text color="gray"> ({displaySubtitle})</Text>
        )}
      </Box>
      <Box>
        <Text color="gray">{'─'.repeat(width)}</Text>
      </Box>
    </Box>
  );
}
