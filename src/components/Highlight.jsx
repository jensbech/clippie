import React from 'react';
import { Text } from 'ink';

export function Highlight({ text, term, color }) {
  if (!term) return <Text color={color}>{text}</Text>;

  const parts = [];
  const lower = text.toLowerCase();
  const needle = term.toLowerCase();
  let last = 0;

  while (true) {
    const idx = lower.indexOf(needle, last);
    if (idx === -1) break;
    if (idx > last) {
      parts.push(<Text key={last} color={color}>{text.slice(last, idx)}</Text>);
    }
    parts.push(
      <Text key={`h${idx}`} backgroundColor="#d2a8ff" color="#000000">
        {text.slice(idx, idx + needle.length)}
      </Text>
    );
    last = idx + needle.length;
  }

  if (last < text.length) {
    parts.push(<Text key={last} color={color}>{text.slice(last)}</Text>);
  }

  return <Text>{parts}</Text>;
}

export function HighlightLine({ text, term, wrap: wrapMode = "wrap" }) {
  if (!term) return <Text wrap={wrapMode}>{text}</Text>;

  const parts = [];
  const lower = text.toLowerCase();
  const needle = term.toLowerCase();
  let last = 0;

  while (true) {
    const idx = lower.indexOf(needle, last);
    if (idx === -1) break;
    if (idx > last) {
      parts.push(<Text key={last}>{text.slice(last, idx)}</Text>);
    }
    parts.push(
      <Text key={`h${idx}`} backgroundColor="#d2a8ff" color="#000000">
        {text.slice(idx, idx + needle.length)}
      </Text>
    );
    last = idx + needle.length;
  }

  if (last < text.length) {
    parts.push(<Text key={last}>{text.slice(last)}</Text>);
  }

  return <Text wrap={wrapMode}>{parts}</Text>;
}
