def run(client, params):
    limit = int(params.get('limit', [10])[0])
    return client.query(
        '''      SELECT
            language,
            COUNT(DISTINCT user) AS number_of_users
        FROM userLanguages
        GROUP BY
            language
        ORDER BY
            number_of_users DESC
        LIMIT {limit}
        ''',
        {
            "limit": limit
        }
    )