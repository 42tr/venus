package jwt

import (
	"errors"
	"fmt"
	"venus/config"

	"github.com/dgrijalva/jwt-go"
	"github.com/gin-gonic/gin"
)

// 解析token
func parseToken(tokenString string) (jwt.MapClaims, error) {
	token, err := jwt.Parse(tokenString, func(token *jwt.Token) (any, error) {
		// Don't forget to validate the alg is what you expect:
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("Unexpected signing method: %v", token.Header["alg"])
		}

		// hmacSampleSecret is a []byte containing your secret, e.g. []byte("my_secret_key")
		return []byte(config.SECRETKEY), nil
	})
	if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
		return claims, nil
	} else {
		return nil, err
	}
}

func Get(c *gin.Context) (uint, error) {
	tokenString, err := c.Cookie(config.HEADER)
	if len(tokenString) == 0 {
		return 0, errors.New("已登出")
	}
	fmt.Println(tokenString)
	if err != nil {
		return 0, err
	}
	m, err := parseToken(tokenString)
	if err != nil {
		return 0, err
	}
	return uint(m["uid"].(float64)), nil
}

func Exp(c *gin.Context) {
	c.SetCookie(config.HEADER, "", 0, "/", "", false, true)
}
