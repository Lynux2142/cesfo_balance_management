from os import getenv
from bs4 import BeautifulSoup
import requests

URL_BASE = "https://www.e-chargement.com/default.Asp"
URL = "https://www.e-chargement.com/identif_badge.Asp"
BADGE_DIV = getenv("BADGE_DIV")
BADGE_NUMBER = getenv("BADGE_NUMBER")
BADGE_NAME = getenv("BADGE_NAME")
FILE_PATH = "/data/balance_history.txt"

def main():
    session = requests.Session()

    headers_base = {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0',
        'Accept-Language': 'en-US,en;q=0.9',
    }
    session.get(URL_BASE, headers=headers_base)

    headers_post = headers_base.copy()
    headers_post.update({
        'Content-Type': 'application/x-www-form-urlencoded',
        'Origin': 'https://www.e-chargement.com',
        'Referer': URL_BASE,
    })

    data = {
        "badge_div": BADGE_DIV,
        "badge_number": BADGE_NUMBER,
        "badge_nom": BADGE_NAME,
    }

    response = session.post(URL, data=data, allow_redirects=True, headers=headers_post)
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
