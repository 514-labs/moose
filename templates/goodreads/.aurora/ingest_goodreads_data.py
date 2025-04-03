import os
import json
import pandas as pd
import requests
from kaggle.api.kaggle_api_extended import KaggleApi
from pathlib import Path
import numpy as np

# Set up Kaggle credentials
os.environ['KAGGLE_USERNAME'] = 'johananottensooser'
os.environ['KAGGLE_KEY'] = '65878c5a89043c70aaa11cc94db4c81a'

def clean_string(value):
    """Clean and escape string values"""
    if pd.isna(value) or value is None:
        return ""
    # Convert to string and strip whitespace
    value = str(value).strip()
    # Replace any problematic characters
    value = value.replace('\n', ' ').replace('\r', ' ')
    return value

def clean_number(value, default=0):
    """Clean and validate numeric values"""
    try:
        if pd.isna(value) or value is None:
            return default
        return float(value) if isinstance(value, str) else value
    except (ValueError, TypeError):
        return default

def clean_book_id(value):
    """Clean and validate bookID as number"""
    try:
        if pd.isna(value) or value is None:
            return 0
        # Remove any non-numeric characters and convert to int
        cleaned = ''.join(filter(str.isdigit, str(value)))
        return int(cleaned) if cleaned else 0
    except (ValueError, TypeError):
        return 0

def download_dataset():
    """Download the Goodreads dataset from Kaggle"""
    api = KaggleApi()
    api.authenticate()
    
    print("Downloading dataset from Kaggle...")
    api.dataset_download_files('jealousleopard/goodreadsbooks', path='.aurora', unzip=True)
    return '.aurora/books.csv'

def display_sample_data(df):
    """Display a sample row of data in both raw and cleaned format"""
    print("\n=== Sample Raw Data ===")
    sample_raw = df.iloc[0].to_dict()
    print(json.dumps(sample_raw, indent=2))
    
    print("\n=== Sample Cleaned Data ===")
    sample_cleaned = {
        "bookID": clean_book_id(sample_raw['bookID']),
        "title": clean_string(sample_raw['title']),
        "authors": clean_string(sample_raw['authors']),
        "average_rating": clean_number(sample_raw['average_rating'], 0.0),
        "isbn": clean_string(sample_raw['isbn']),
        "isbn13": clean_string(sample_raw['isbn13']),
        "language_code": clean_string(sample_raw['language_code']),
        "num_pages": int(clean_number(sample_raw['num_pages'], 0)),
        "ratings_count": int(clean_number(sample_raw['ratings_count'], 0)),
        "text_reviews_count": int(clean_number(sample_raw['text_reviews_count'], 0)),
        "publication_date": clean_string(sample_raw['publication_date']),
        "publisher": clean_string(sample_raw['publisher'])
    }
    print(json.dumps(sample_cleaned, indent=2))
    print("\nContinue with data processing? (y/n)")
    if input().lower() != 'y':
        print("Aborting data processing")
        exit(0)

def process_and_ingest_data(csv_path):
    """Process the CSV file and send data to ingestion API"""
    print("Reading CSV file...")
    # Read CSV with proper escaping
    df = pd.read_csv(csv_path, 
                    escapechar='\\',
                    encoding='utf-8',
                    on_bad_lines='warn',
                    dtype={
                        'bookID': str,  # Read as string first to handle any non-numeric characters
                        'title': str,
                        'authors': str,
                        'average_rating': float,
                        'isbn': str,
                        'isbn13': str,
                        'language_code': str,
                        '  num_pages': float,  # Using float to handle potential NaN values
                        'ratings_count': float,
                        'text_reviews_count': float,
                        'publication_date': str,
                        'publisher': str
                    })
    
    # Clean column names
    df.columns = df.columns.str.strip()
    
    # Display sample data and ask for confirmation
    display_sample_data(df)
    
    # Convert DataFrame to list of dictionaries
    books = df.to_dict('records')
    
    # Process data in batches of 1000
    batch_size = 1000
    total_batches = len(books) // batch_size + (1 if len(books) % batch_size > 0 else 0)
    
    print(f"Processing {len(books)} books in {total_batches} batches...")
    
    for i in range(0, len(books), batch_size):
        batch = books[i:i+batch_size]
        
        # Clean and format the data according to schema
        formatted_books = []
        for book in batch:
            try:
                formatted_book = {
                    "bookID": clean_book_id(book['bookID']),
                    "title": clean_string(book['title']),
                    "authors": clean_string(book['authors']),
                    "average_rating": clean_number(book['average_rating'], 0.0),
                    "isbn": clean_string(book['isbn']),
                    "isbn13": clean_string(book['isbn13']),
                    "language_code": clean_string(book['language_code']),
                    "num_pages": int(clean_number(book['num_pages'], 0)),
                    "ratings_count": int(clean_number(book['ratings_count'], 0)),
                    "text_reviews_count": int(clean_number(book['text_reviews_count'], 0)),
                    "publication_date": clean_string(book['publication_date']),
                    "publisher": clean_string(book['publisher'])
                }
                # Skip records with invalid bookID
                if formatted_book["bookID"] == 0:
                    print(f"Skipping book with invalid ID: {book.get('title', 'Unknown')}")
                    continue
                formatted_books.append(formatted_book)
            except Exception as e:
                print(f"Error processing book: {book.get('title', 'Unknown')}")
                print(f"Error details: {str(e)}")
                continue
        
        if not formatted_books:
            print(f"Skipping empty batch {(i//batch_size)+1}/{total_batches}")
            continue
            
        # Send books individually
        success_count = 0
        error_count = 0
        for book in formatted_books:
            try:
                response = requests.post(
                    'http://localhost:4000/ingest/books',
                    json=book,  # Send single book object
                    headers={'Content-Type': 'application/json'}
                )
                response.raise_for_status()
                success_count += 1
            except requests.exceptions.RequestException as e:
                print(f"Error ingesting book {book.get('title', 'Unknown')}: {str(e)}")
                error_count += 1
                # Save failed book for retry
                error_file = f'.aurora/failed_book_{book["bookID"]}.json'
                with open(error_file, 'w') as f:
                    json.dump(book, f)
                print(f"Failed book saved to {error_file}")
        
        print(f"Batch {(i//batch_size)+1}/{total_batches}: Successfully ingested {success_count} books, {error_count} failures")

def cleanup(csv_path):
    """Clean up downloaded files"""
    try:
        os.remove(csv_path)
        print("Cleaned up downloaded files")
    except Exception as e:
        print(f"Error during cleanup: {str(e)}")

def main():
    # Create .aurora directory if it doesn't exist
    Path('.aurora').mkdir(exist_ok=True)
    
    try:
        # Download and process the dataset
        csv_path = download_dataset()
        process_and_ingest_data(csv_path)
        cleanup(csv_path)
        print("Data ingestion completed successfully!")
    except Exception as e:
        print(f"An error occurred: {str(e)}")

if __name__ == "__main__":
    main() 