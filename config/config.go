package config

import (
	"os"

	"github.com/joho/godotenv"
)

var SECRETKEY string
var HEADER string
var AUTH_URL string

func init() {
	err := godotenv.Load()
	if err != nil {
		panic("Error loading .env file")
	}
	SECRETKEY = os.Getenv("SECRETKEY")
	if SECRETKEY == "" {
		panic("SECRETKEY is not set")
	}
	HEADER = os.Getenv("HEADER")
	if HEADER == "" {
		panic("HEADER is not set")
	}
	AUTH_URL = os.Getenv("AUTH_URL")
	if AUTH_URL == "" {
		panic("AUTH_URL is not set")
	}
}
