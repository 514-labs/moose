# Goodreads Data Models

## Installation and Setup

From the project home directory

1. **Install Moose / Aurora** 
```bash
bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,aurora
```

2. **Initiate your project**
```bash
aurora init books goodreads
```

3. **Install Dependencies**
```bash
npm install
```

4. **Configure Kaggle**
- Add your Kaggle Settings file to the home directory of this project
- For more information, see https://www.kaggle.com/docs/api

5. **Start Development Server**
```bash
moose dev
```

6. **Run Data Ingestion**
```bash
python .aurora/ingest_goodreads_data.py
```

## Data Models Overview

This project contains data models for storing and processing Goodreads book information. Below are the available ingestion APIs and their corresponding schemas.

## Ingestion APIs

### 1. Single Book Ingestion
- **Endpoint**: `http://localhost:4000/ingest/books`
- **Format**: JSON
- **Method**: POST
- **Deduplication**: Enabled (based on bookID)

### 2. Batch Books Ingestion
- **Endpoint**: `http://localhost:4000/ingest/goodreads_books`
- **Format**: JSON Array
- **Method**: POST
- **Deduplication**: Enabled (based on bookID)
- **Stream Configuration**:
  - Parallelism: 4
  - Retention Period: 24 hours

## Data Schemas

Both ingestion endpoints accept data with the following schema:

```typescript
interface BookSchema {
  bookID: string;      // Primary key
  title: string;       // Book title
  authors: string;     // Book author(s)
  average_rating: number;  // Average rating (0-5)
  isbn: string;        // 10-digit ISBN
  isbn13: string;      // 13-digit ISBN
  language_code: string;   // Book language code (e.g., 'eng')
  num_pages: number;   // Number of pages
  ratings_count: number;   // Total number of ratings
  text_reviews_count: number;  // Total number of text reviews
  publication_date: string;    // Publication date
  publisher: string;   // Publisher name
}
```

## Usage Examples

### Single Book Ingestion
```bash
curl -X POST http://localhost:4000/ingest/books \
  -H "Content-Type: application/json" \
  -d '{
    "bookID": "1",
    "title": "Harry Potter and the Half-Blood Prince",
    "authors": "J.K. Rowling",
    "average_rating": 4.57,
    "isbn": "0439785960",
    "isbn13": "9780439785969",
    "language_code": "eng",
    "num_pages": 652,
    "ratings_count": 2095690,
    "text_reviews_count": 27591,
    "publication_date": "2006-09-16",
    "publisher": "Scholastic Inc."
  }'
```

### Batch Books Ingestion
```bash
curl -X POST http://localhost:4000/ingest/goodreads_books \
  -H "Content-Type: application/json" \
  -d '[
    {
      "bookID": "1",
      "title": "Harry Potter and the Half-Blood Prince",
      "authors": "J.K. Rowling",
      "average_rating": 4.57,
      "isbn": "0439785960",
      "isbn13": "9780439785969",
      "language_code": "eng",
      "num_pages": 652,
      "ratings_count": 2095690,
      "text_reviews_count": 27591,
      "publication_date": "2006-09-16",
      "publisher": "Scholastic Inc."
    },
    // ... more books ...
  ]'
```

## Notes
- The `books` endpoint is optimized for single record ingestion
- The `goodreads_books` endpoint is optimized for batch ingestion with parallel processing
- Both endpoints perform deduplication based on the `bookID` field
- Dates should be provided in YYYY-MM-DD format
- All numeric fields should be provided as numbers, not strings 
