import { ConsumptionApi } from "@514labs/moose-lib";
import { tags } from "typia";

/**
 * Parameters for the getBooksByYearSortedByRating API
 */
interface BooksByYearParams {
  /** The publication year to filter by (e.g., 2006) */
  year: number & tags.Type<"int32"> & tags.Minimum<1800> & tags.Maximum<2100>;
}

/**
 * Book data structure returned by the API
 */
interface Book {
  title: string;
  authors: string;
  rating: number;
  ratings_count: number;
  reviews_count: number;
  publication_date: string;
  publisher: string;
  pages: number;
}

/**
 * API that returns a list of books for a given year, sorted by their average rating in descending order.
 * Only includes books with a significant number of ratings (>1000) to ensure reliability.
 */
export const getBooksByYearSortedByRating =
  new ConsumptionApi<BooksByYearParams>(
    "getBooksByYearSortedByRating",
    async (params, utils) => {
      const { client, sql } = utils;

      try {
        // Execute the query with parameterized input to prevent SQL injection
        const result = await client.query.execute(sql`
        WITH filtered_books AS (
          SELECT 
            title,
            authors,
            ROUND(CAST(average_rating AS Float64), 2) as rating,
            CAST(ratings_count AS Int64) as ratings_count,
            CAST(text_reviews_count AS Int64) as reviews_count,
            publication_date,
            publisher,
            CAST(num_pages AS Int32) as pages
          FROM books
          WHERE toYear(parseDateTime64BestEffort(publication_date)) = ${params.year}
            AND ratings_count >= 1000
        )
        SELECT *
        FROM filtered_books
        ORDER BY rating DESC, ratings_count DESC
      `);

        // Process the result set and convert to the expected format
        const books: Book[] = await result.json();

        return {
          success: true,
          count: books.length,
          books: books,
        };
      } catch (error) {
        // Basic error handling
        console.error("Error fetching books by year:", error);
        return {
          success: false,
          error: "Failed to fetch books",
          details: error instanceof Error ? error.message : "Unknown error",
        };
      }
    },
  );
