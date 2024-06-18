import requests
from os import getenv
from bs4 import BeautifulSoup
from os.path import exists
from urllib.parse import urlencode
from datetime import datetime, timedelta

URL = "https://www.e-chargement.com/identif_badge.Asp"
BADGE_DIV = getenv("BADGE_DIV")
BADGE_NUMBER = getenv("BADGE_NUMBER")
BADGE_NAME = getenv("BADGE_NAME")
FILE_PATH = "/data/balance_history.txt"

def estimate_meal_price(previous_balance, current_balance, previous_incoming_credit):
    meal_price = previous_balance - (current_balance - previous_incoming_credit)
    return (meal_price)

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
    current_date_obj = datetime.strptime(data[2].text.split(" ")[0], '%d/%m/%y')
    current_date_obj -= timedelta(days=1)
    current_date_str = current_date_obj.strftime('%d/%m/%y')
    previous_balance = current_balance
    incoming_credit = data[3].text.replace(",", ".")
    previous_incoming_credit = f"0.0 \N{euro sign}"
    if exists(FILE_PATH):
        with open(FILE_PATH, "r") as f:
            last_history_line = f.readlines()[-1]
            previous_balance = last_history_line.split(";")[0]
            previous_incoming_credit = last_history_line.split(";")[2]
    else:
        with open(FILE_PATH, "w") as f:
            f.write("Account Balance;Meal Price;Incoming Credit;Date\n")
    meal_price = estimate_meal_price(
        float(previous_balance.split(" ")[0]),
        float(current_balance.split(" ")[0]),
        float(previous_incoming_credit.split(" ")[0])
    )
    with open(FILE_PATH, "a") as f:
        f.write(f"{current_balance};{meal_price:.2f} \N{euro sign};{incoming_credit};{current_date_str}\n")

if __name__ == "__main__":
    main()
