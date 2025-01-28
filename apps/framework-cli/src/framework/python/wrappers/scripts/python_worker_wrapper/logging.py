from logging import getLogger, INFO, StreamHandler, Formatter
import sys

log = getLogger(__name__)
log.setLevel(INFO)  # Set the logger level to INFO

handler = StreamHandler(sys.stdout)
handler.setLevel(INFO)
handler.setFormatter(Formatter("%(levelname)s | %(name)s | %(message)s"))
log.addHandler(handler)