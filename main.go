// This file is a helper to locally run the serverless function defined in
// `api/`, for development purposes. It shouldn't usually be relevant, unless
// you want to work on the "share notebooks as GitHub Gists" feature in your
// local development environment.

package main

import (
	"fmt"
	"log"
	"net/http"

	"percival.ink/api"
)

func main() {
	http.HandleFunc("/api", api.Handler)

	fmt.Println("Listening at http://localhost:3030/api")
	err := http.ListenAndServe(":3030", nil)

	if err != nil {
		log.Fatal(err)
	}
}
