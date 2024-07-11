WITH ['(?i)topics'] AS patterns
SELECT
    date,
    message,
    source,
    multiFuzzyMatchAllIndices(message, 2, patterns) AS indices
FROM ParsedLogs_0_5
WHERE length(indices) > 0


SELECT *, COUNT(*) OVER() AS totalRowCount FROM ParsedLogs_0_5 LIMIT 10 OFFSET 10