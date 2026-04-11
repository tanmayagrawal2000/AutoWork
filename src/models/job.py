from dataclasses import dataclass
from datetime import date
from typing import Optional

@dataclass
class Job:
    name: str
    id: str
    posting_date: Optional[date]
    location_city: str
    location_country: str
