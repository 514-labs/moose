interface Aggregation {
  select: string;
  orderBy: string;
}

export default {
  select: ` 
      SELECT 
          countState() as errorCount,
          toStartOfDay(observedTimestamp) as date
      FROM Log_0_0
      WHERE severityText = 'ERROR' 
      GROUP BY toStartOfDay(observedTimestamp)
    `,
  orderBy: "date",
} satisfies Aggregation as Aggregation;
