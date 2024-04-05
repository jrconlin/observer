# Consume a stream of data looking for high frequency occurrences of a given data element.

# yes, I know there are combined versions of these, but I wanted to skip additional
# imports if I could.
import argparse
import configparser
import re
import logging
import sys

from misra_gries import Misra_Gries, MG_Error


class Scanner_Error(Exception):
    pass


class Boring(Exception):
    pass


class Settings(object):
    pattern: re.Pattern | None = None
    source: str
    log: str = "WARN"

    def __init__(cls):
        parser = argparse.ArgumentParser(
            prog="Observer", description=("Find interesting things and count them.")
        )
        parser.add_argument(
            "-c", "--config", help="Configuration INI file.", default="config.ini"
        )
        parser.add_argument(
            "-l", "--logging", help="Logging level (DEBUG, INFO, WARN, etc.)"
        )
        parser.add_argument(
            "-s", "--source", help='Source to read from. Use "-" for (STDIN)'
        )
        parser.add_argument(
            "--pattern",
            help="Regex search pattern for interesting data (matches group 1)",
        )
        parser.add_argument(
            "--count",
            help="Display this many in the report",
        )
        parser.add_argument("--count", default="10", help="Report this many top hits")
        args = parser.parse_args()
        config = configparser.ConfigParser()
        config["DEFAULT"] = {
            "pattern": "",
            "source": "-",
        }
        config.read(args.config)
        cls.source = args.source or config["DEFAULT"].get("source", "-")
        cls.log = logging._nameToLevel.get(
            args.logging.upper() or config["DEFAULT"].get("log", "WARN").upper()
        )
        cls.pattern = args.pattern or config["DEFAULT"].get("pattern", "(.*) ")
        cls.count = args.count or config["DEFAULT"].get("count", "10")


class Scanner(object):
    def __init__(
        cls,
        settings: Settings,
        collector: Misra_Gries = Misra_Gries(),  # in case I want to try different collectors.
    ):
        cls.regex = re.compile(settings.pattern)
        cls.collector = collector
        pass

    def scan(self, line: bytearray) -> bytearray:
        """scan line for intersting thing"""
        logger = logging.getLogger(__name__)
        interest = None
        if self.regex:
            try:
                if isinstance(line, bytes):
                    line = line.decode("utf-8")
                groups = self.regex.search(line)
                logger.debug(f"{groups}")
                interest = groups.group(1)
            except AttributeError:
                pass
        if not interest or len(interest) == 0:
            raise Boring()
        return interest


class Process(object):
    source: str

    def __init__(cls, settings: Settings, scanner: Scanner):
        logger = logging.getLogger(__name__)
        if settings.source == "-":
            logger.info("Reading from STDIN")
            cls.source = sys.stdin
        else:
            logger.info(f"Reading from {settings.source}")
            cls.source = open(settings.source, "rb")
        cls.scanner = scanner.scan
        cls.collector = Misra_Gries()
        cls.count = settings.count

    def report(self, count: int = 10):
        """Report interesting counts"""
        logger = logging.getLogger(__name__)
        print(f"Top {count} elements:")
        items = self.collector.top(count)
        if len(items) < count:
            logger.debug("Not enough yet...")
            return
        for i in range(0, count):
            (item, num) = items[i]
            print(f"#{i+1}\t {item:<48} {num}")

    def start(self):
        """Open a stream and begin iterating through on read."""
        counter = 0
        while True:
            line = self.source.readline()
            if len(line) == 0:
                break
            try:
                interesting = self.scanner(line)
                self.collector.insert(interesting)
            except Boring:
                continue
            except Scanner_Error as ex:
                logger.error(ex)
                break
        self.report(self.count)


if __name__ == "__main__":

    settings = Settings()
    logger = logging.getLogger(__name__)
    logging.basicConfig(level=settings.log)
    logger.info("Starting...")
    if settings.pattern:
        logger.debug(f"Scanning for {settings.pattern}")

    Process(settings, Scanner(settings)).start()
