from os import getenv
from bs4 import BeautifulSoup
import requests

URL = "https://www.e-chargement.com/identif_badge.Asp"
BADGE_DIV = getenv("BADGE_DIV")
BADGE_NUMBER = getenv("BADGE_NUMBER")
BADGE_NAME = getenv("BADGE_NAME")
FILE_PATH = "/data/balance_history.txt"

def main():
    session = requests.Session()

    data = {
        "badge_div": BADGE_DIV,
        "badge_number": BADGE_NUMBER,
        "badge_nom": BADGE_NAME,
    }

    response = session.post(URL, data=data, allow_redirects=True)
    response.encoding = "utf-8"

    parsed_html = BeautifulSoup(response.text, "html.parser")
    data = parsed_html.find_all("td", {"class": "bold"})

    incoming_credit = data[3].text.replace(",", ".")
    if not int(incoming_credit.split(" ")[0]):
        return 0
    with open(FILE_PATH, "r") as f:
        lines = f.readlines()
        last_line = lines[-1]
        data = last_line.split(";")
        data[2] = incoming_credit
        del lines[-1]
        lines.append(";".join(data))
    with open(FILE_PATH, "w") as f:
        f.write("".join(lines))

if __name__ == "__main__":
    main()
