from moose_lib import Key
from dataclasses import dataclass
from datetime import date
from typing import Optional

@dataclass
class Sender:
    login: Key[str]
    repos_url: str

# @dataclass
# class Repository:
#     html_url: str
#     url: str

@dataclass
class RawStarEvent:
    starred_at: Optional[str]
    action: Key[str]
    sender: Sender
    # repository: Repository

@dataclass
class LanguageCount:
    language: Key[str]
    bytes: int

@dataclass
class ProcessedStarEvent:
    starred_at: Key[str]
    username: str
    languages: list[LanguageCount]
