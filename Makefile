.DEFAULT_GOAL := init
.PHONY: init

UID = $(shell id -u)
DATA_PATH= $(shell pwd)/data/

init:
	@echo "UID=$(UID)" > .env
	@read -p "Badge name (Lastname Firstname): " BADGE_NAME;echo BADGE_NAME=$$BADGE_NAME >> .env
	@read -p "Badge number: " BADGE_NUMBER;echo BADGE_NUMBER=$$BADGE_NUMBER >> .env
	@echo "DATA=$(DATA_PATH)" >> .env
	mkdir -p data
	docker compose up -d
	docker compose run -T --build update_balance_history
	docker compose run -T --build check_incoming_credit

fclean:
	docker compose down --rmi all --volumes --remove-orphans
	rm .env
