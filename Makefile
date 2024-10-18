.PHONY: fixtures

help:
	echo "HELP"

migrate:
	@cat migrations/*.sql | psql -U charon charon

fixtures:
	@cat fixtures/*.sql | psql -U charon charon

