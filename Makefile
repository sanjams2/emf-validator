
AWS_REGION=us-east-1
AWS_PROFILE=emf-validator
STACK_NAME=emf-validator-website

build:
	cargo build

wasm-pack:
	wasm-pack build --target web --release --out-dir site/pkg

install-tailwind:
	curl -sL https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-x64 -o bin/tailwindcss
	chmod +x bin/tailwindcss

tailwind:
	./bin/tailwindcss -i site/styles/input.css -o site/styles/main.min.css --config etc/tailwind.config.js --minify

release: wasm-pack

tailwind-watch:
	./bin/tailwindcss -i site/styles/input.css -o site/styles/main.min.css --config etc/tailwind.config.js --watch --minify

local:
	python3 -m http.server --directory site

cfn-init:
	aws --profile $(AWS_PROFILE) --region $(AWS_REGION) cloudformation create-stack \
		--stack-name $(STACK_NAME) \
		--template-body file://configuration/template.yaml \
		--parameter \
			ParameterKey="DomainName",ParameterValue=$(DOMAIN_NAME)

cfn-update:
	aws --profile $(AWS_PROFILE) --region $(AWS_REGION) cloudformation update-stack \
		--stack-name $(STACK_NAME) \
		--template-body file://configuration/template.yaml \
		--parameter \
			ParameterKey="DomainName",ParameterValue=$(DOMAIN_NAME)

invalidate:
	aws --profile $(AWS_PROFILE) cloudfront create-invalidation \
		--distribution-id $(shell aws --region $(AWS_REGION) --profile $(AWS_PROFILE) cloudformation describe-stacks --stack-name $(STACK_NAME) --query "Stacks[0].Outputs[?OutputKey=='CloudfrontDistributionId'].OutputValue" --output text) \
		--paths "/*"

release: invalidate
	aws --profile $(AWS_PROFILE) s3 cp site s3://$(DOMAIN_NAME) --recursive --cache-control max-age=86400
