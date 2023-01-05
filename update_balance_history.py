import requests
from os import getenv
from bs4 import BeautifulSoup
from os.path import exists
from urllib.parse import urlencode

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

    headers = {
        "Content-type": "application/x-www-form-urlencoded"
    }

    response = session.post(URL, headers=headers, data=urlencode(data), allow_redirects=True)
    response.encoding = response.apparent_encoding

    parsed_html = BeautifulSoup(response.text, "html.parser")
    data = parsed_html.find_all("td", {"class": "bold"})

    current_balance = data[1].text.replace(",", ".")
    current_date = data[2].text.split(" ")[0]
    if exists(FILE_PATH):
        with open(FILE_PATH, "r") as f:
            previous_balance = f.readlines()[-1].split(";")[0]
    else:
        with open(FILE_PATH, "w") as f:
            f.write("Account Balance;Meal Price;Date\n")
        previous_balance = current_balance
    meal_price = float(previous_balance.split(" ")[0]) - float(current_balance.split(" ")[0])
    with open(FILE_PATH, "a") as f:
        f.write(f"{current_balance};{meal_price:.2f} \N{euro sign};{current_date}\n")

if __name__ == "__main__":
    main()
