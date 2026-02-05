import React, { useState, useEffect, useCallback, memo } from 'react';
import { Box, Text, useApp, useInput, useStdout } from 'ink';
import Header from './components/Header.jsx';
import StatusBar, { KeyHint } from './components/StatusBar.jsx';
import ClickableRow from './components/ClickableRow.jsx';
import PreviewPanel from './components/PreviewPanel.jsx';
import { getEntries } from './lib/db.js';
import { getDbPath } from './lib/config.js';
import { relativeDate } from './lib/dates.js';
import { Highlight } from './components/Highlight.jsx';

const DATE_COL = 8;

function pad(str, len) {
  if (str.length >= len) return str.slice(0, len);
  return str + ' '.repeat(len - str.length);
}

function truncate(str, len) {
  if (str.length <= len) return pad(str, len);
  return str.slice(0, len - 1) + '…';
}

const EntryRow = memo(function EntryRow({ preview, dateStr, isSelected, width, highlight }) {
  const contentWidth = width - 2 - DATE_COL - 1;
  const line = truncate(preview, contentWidth);
  const date = pad(dateStr, DATE_COL);
  const color = isSelected ? 'cyan' : undefined;

  return (
    <Box width={width}>
      <Text wrap="truncate" color={color}>
        {isSelected ? '>' : ' '}{' '}
        <Highlight text={line} term={highlight} color={color} />
      </Text>
      <Text wrap="truncate" color="gray"> {date}</Text>
    </Box>
  );
});

export default function App({ onSelect }) {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const terminalWidth = stdout?.columns || 80;
  const terminalHeight = stdout?.rows || 24;

  const [entries, setEntries] = useState(() => getEntries());
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [scrollOffset, setScrollOffset] = useState(0);
  const [filterText, setFilterText] = useState('');
  const [isFiltering, setIsFiltering] = useState(false);
  const [message, setMessage] = useState(null);

  // Get database path for display
  const dbPath = getDbPath();
  const dbPathShort = dbPath.split('/').slice(-2).join('/');

  const usableHeight = Math.max(5, Math.floor((terminalHeight - 6) * 0.95));
  const leftWidth = Math.floor((terminalWidth - 1) / 2);
  const rightWidth = terminalWidth - leftWidth - 1;

  const loadEntries = useCallback(() => {
    setEntries(getEntries());
    setMessage('Refreshed');
  }, []);

  useEffect(() => {
    if (message) {
      const timer = setTimeout(() => setMessage(null), 2000);
      return () => clearTimeout(timer);
    }
  }, [message]);

  const filteredEntries = filterText
    ? entries.filter(e => e.content.toLowerCase().includes(filterText.toLowerCase()))
    : entries;

  const visibleEntries = filteredEntries.slice(scrollOffset, scrollOffset + usableHeight);
  const currentEntry = filteredEntries[selectedIndex] || null;

  const selectEntry = useCallback((entry) => {
    onSelect(entry.content);
    exit();
  }, [onSelect, exit]);

  useInput((input, key) => {
    if (isFiltering) {
      if (key.escape) {
        setIsFiltering(false);
        setFilterText('');
        setSelectedIndex(0);
        setScrollOffset(0);
      } else if (key.return) {
        setIsFiltering(false);
      } else if (key.backspace || key.delete) {
        setFilterText(t => t.slice(0, -1));
        setSelectedIndex(0);
        setScrollOffset(0);
      } else if (input && !key.ctrl && !key.meta) {
        setFilterText(t => t + input);
        setSelectedIndex(0);
        setScrollOffset(0);
      }
      return;
    }

    if (input === 'q' || key.escape) {
      if (filterText) {
        setFilterText('');
        setSelectedIndex(0);
        setScrollOffset(0);
      } else {
        exit();
      }
    } else if (input === '/') {
      setIsFiltering(true);
    } else if (input === 'r') {
      loadEntries();
      setSelectedIndex(0);
      setScrollOffset(0);
    } else if (key.upArrow || input === 'k') {
      setSelectedIndex(i => {
        const newIndex = Math.max(0, i - 1);
        if (newIndex < scrollOffset) {
          setScrollOffset(newIndex);
        }
        return newIndex;
      });
    } else if (key.downArrow || input === 'j') {
      setSelectedIndex(i => {
        const newIndex = Math.min(filteredEntries.length - 1, i + 1);
        if (newIndex >= scrollOffset + usableHeight) {
          setScrollOffset(newIndex - usableHeight + 1);
        }
        return newIndex;
      });
    } else if (key.return) {
      if (currentEntry) {
        selectEntry(currentEntry);
      }
    }
  });

  const handleRowSelect = useCallback((visibleIndex) => {
    setSelectedIndex(scrollOffset + visibleIndex);
  }, [scrollOffset]);

  const handleRowActivate = useCallback((visibleIndex) => {
    const entry = filteredEntries[scrollOffset + visibleIndex];
    if (entry) selectEntry(entry);
  }, [filteredEntries, scrollOffset, selectEntry]);

  const filterInfo = filterText ? ` filter: "${filterText}"` : '';
  const subtitle = `${filteredEntries.length} entries${filterInfo}`;

  return (
    <Box flexDirection="column">
      <Header title="History" subtitle={subtitle} />

      <Box>
        {/* Left panel: list */}
        <Box flexDirection="column" width={leftWidth} height={usableHeight} overflow="hidden" flexShrink={0} flexGrow={0}>
          {filteredEntries.length === 0 ? (
            <Text color="gray">
              {entries.length === 0 ? 'No clipboard history found.' : 'No matches.'}
            </Text>
          ) : (
            visibleEntries.map((entry, index) => {
              const actualIndex = scrollOffset + index;
              const isSelected = actualIndex === selectedIndex;
              const preview = entry.content.replace(/[\n\r]+/g, '↵').replace(/\s+/g, ' ');
              const dateStr = relativeDate(entry.lastCopied);

              return (
                <ClickableRow
                  key={entry.id}
                  index={index}
                  onSelect={handleRowSelect}
                  onActivate={handleRowActivate}
                >
                  <EntryRow
                    preview={preview}
                    dateStr={dateStr}
                    isSelected={isSelected}
                    width={leftWidth}
                    highlight={filterText}
                  />
                </ClickableRow>
              );
            })
          )}
        </Box>

        {/* Divider */}
        <Box flexDirection="column" width={1} height={usableHeight} flexShrink={0} flexGrow={0}>
          <Text color="gray">{'│\n'.repeat(usableHeight).trimEnd()}</Text>
        </Box>

        {/* Right panel: preview */}
        <PreviewPanel entry={currentEntry} width={rightWidth} height={usableHeight} highlight={filterText} />
      </Box>

      <StatusBar
        rightContent={!isFiltering && (
          <Text color="gray">DB: {dbPathShort}</Text>
        )}
      >
        {isFiltering ? (
          <Text>
            <Text color="yellow">Filter: </Text>
            <Text>{filterText}</Text>
            <Text color="gray">_</Text>
            <Text color="gray">  (Enter to confirm, Esc to cancel)</Text>
          </Text>
        ) : (
          <>
            <KeyHint keyName="Enter" description=" copy" />
            <KeyHint keyName="/" description=" filter" onClick={() => setIsFiltering(true)} />
            <KeyHint keyName="r" description="efresh" onClick={loadEntries} />
            <KeyHint keyName="q" description="uit" onClick={() => exit()} />
          </>
        )}
      </StatusBar>
    </Box>
  );
}
