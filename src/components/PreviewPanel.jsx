import React from 'react';
import { Box, Text } from 'ink';
import { prettyDate } from '../lib/dates.js';
import { HighlightLine } from './Highlight.jsx';

export default function PreviewPanel({ entry, width, height, highlight }) {
  if (!entry) {
    return (
      <Box flexDirection="column" width={width} height={height} paddingLeft={1} flexShrink={0} flexGrow={0}>
        <Text color="gray">No entry selected</Text>
      </Box>
    );
  }

  const dateStr = prettyDate(entry.lastCopied);

  return (
    <Box flexDirection="column" width={width} height={height} paddingLeft={1} overflow="hidden" flexShrink={0} flexGrow={0}>
      <Box>
        <Text backgroundColor="#161b22" bold color="white">
          {'  '}{dateStr}{'  '}
        </Text>
      </Box>
      <Text>{''}</Text>
      <HighlightLine text={entry.content} term={highlight} />
    </Box>
  );
}
