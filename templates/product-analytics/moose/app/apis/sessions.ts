export interface QueryParams {
  limit: string;
  duration: string;
}

export default async function handle(
  { limit = "100", duration = "120" }: QueryParams,
  { client, sql },
) {
  const limitNum = parseInt(limit);
  const durationNum = parseInt(duration);

  return client.query(
    sql`
      SELECT * FROM sessions
      WHERE duration > ${durationNum}
       LIMIT ${limitNum};
  `,
  );
}
