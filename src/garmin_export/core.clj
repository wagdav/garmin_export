(ns garmin-export.core
  (:import [org.openqa.selenium By WebElement]
           [org.openqa.selenium.chrome ChromeDriver]))

(comment
  (System/setProperty "webdriver.chrome.driver", "/nix/store/xgr97saj4069pygxsr2vlv0ivpdiyjkh-undetected-chromedriver-135.0.7049.96/bin/undetected-chromedriver")
  (def driver (ChromeDriver.))
  (.quit driver)

  (.get driver "https://connect.garmin.com/signin")
  (doto (.findElement driver (By/id "email"))
        (.sendKeys (into-array CharSequence ["email"])))
  (doto (.findElement driver (By/id "password"))
        (.sendKeys (into-array CharSequence [(.trim (slurp "g.txt"))])))
  (doto (.findElement driver (By/cssSelector "[type='submit']"))
        (.click))

  (.findElement driver (By/xpath "//button[normalize-space()=\"Sign In\"]")))
