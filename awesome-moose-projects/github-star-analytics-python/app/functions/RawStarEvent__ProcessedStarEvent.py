
# Add your models & start the development server to import these types
from app.datamodels.models import RawStarEvent, ProcessedStarEvent, LanguageCount

from moose_lib import StreamingFunction
from typing import Optional
from datetime import datetime
import requests
import os

def fn(source: RawStarEvent) -> Optional[ProcessedStarEvent]:
    if source.starred_at is None:
        return None
    print('source.sender', source.sender)
    languages = get_user_language(source.sender['repos_url'])
    print(languages)

    return ProcessedStarEvent(
        starred_at=source.starred_at,
        username=source.sender['login'],
        languages=languages,
    )

my_function = StreamingFunction(
    run=fn
)

def get_user_language(repo_url: str) -> list[LanguageCount]:
    print('get_user_language', repo_url)
    repositories = call_github_api(repo_url)
    print('repositories', repositories)
    # Dictionary to store language counts
    language_counts = []
    for repo in repositories:
        # Calls the API per repository to get language counts
        languages = call_github_api(repo['languages_url'])
        print(languages)
        for language, count in languages.items():
            # Sum the counts for each language
            language_counts.append(LanguageCount(language=language, bytes=count))
    return language_counts

def call_github_api(url: str) -> dict:
    response = requests.get(url, auth=('Bearer token ', os.environ['GITHUB_TOKEN']))
    return response.json()