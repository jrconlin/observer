# A minimal implementation of the Misra-Gries frequent element detector.
# _See [Wikipedia](https://en.wikipedia.org/wiki/Misra%E2%80%93Gries_summary) for details_

"""
"""
from collections import Counter


DEFAULT_TTL: int = 60 * 60 * 4
MAX_COLLECTION_COUNT = 32_000


class MG_Error(Exception):
    pass


class Misra_Gries(object):
    # Collection of values
    counter: Counter
    # Reset collection at this interval
    ttl: int
    # number of items to attempt to collect
    limit: int

    def __init__(cls, ttl: int = DEFAULT_TTL, limit: int = 5000):
        cls.counter = Counter()
        cls.limit = limit

    def top(self, count: int | None = 100) -> list[tuple[str, int]]:
        """return the top `count` keys based on value"""
        return self.counter.most_common(count)

    def reset(self):
        """Reset"""
        self.counter.clear()

    def purge(self, count: int | None = None):
        """remove the bottom `count` values. If count == None,
        removes values below median.
        """
        remainder = self.counter.most_common()[: len(self.counter) / 2]
        for item in remainder:
            del self.counter[item[0]]

    def insert(self, item: str) -> bool:
        """Attempts to insert `item`. This causes the value associaed with key(`item`) to be incremented.
        If `item` does not appear in the list, then it is either added, or dropped depeneding if the collections has reached it's limit.
        """
        if item in self.counter or len(self.counter) < self.limit:
            # add the key to build up the base (or bounce up the existing values)
            self.counter[item] += 1
            return True
        else:
            # decrement
            for key in list(self.counter.keys()):
                self.counter[key] -= 1
                if self.counter[key] < 1:
                    del self.counter[key]
            return False
