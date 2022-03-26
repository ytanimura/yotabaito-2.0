REGISTORY  := registry.gitlab.com/ytanimura/yotabaito

ci-push:
	docker login $(REGISTORY)
	docker build -t $(REGISTORY)/ci-container:latest -f ci-container.dockerfile .
	docker push $(REGISTORY)/ci-container:latest

