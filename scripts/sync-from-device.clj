#!/usr/bin/env bb
(ns net.thewagner.garmin
  (:require [clojure.java.io :as io])
  (:require [clojure.java.shell :refer [sh]]))

(def devices
  [{:id "usb-Garmin_FR935_Flash-0:0"
    :name "Garmin Forerunner 935"
    :activities-path ["GARMIN" "ACTIVITY"]}

   {:id "usb-Garmin_GARMIN_Flash-0:0"
    :name "Garmin Edge 530"
    :activities-path ["Garmin" "Activities"]}])


(defn mount-device [device mountpoint]
  (sh "pmount" "-r" device mountpoint))

(defn umount-device [device]
  (sh "pumount" device))

(defn existing-directory [p]
  (when (.exists (io/file p))
    p))

(defn detect-device! []
  (some (fn [d]
          (let [device-path (io/file "/dev/disk/by-id" (:id d))]
            (when (.exists device-path)
              (assoc d :device-path (str device-path)))))
        devices))

(defn with-device-path [device f]
  (let [mountpoint "/media/garmin"
        _ (mount-device device mountpoint)
        result (f mountpoint)
        _ (umount-device device)]
    result))

(let [[dest-dir] *command-line-args*]
  (when (empty? dest-dir)
    (println "Usage: sync-from-device.clj <DEST_DIRECTORY>")
    (System/exit 1))

  (let [device (detect-device!)]
    (if device
      (do
        (println "Syncing files from" (:name device))
        (with-device-path (:device-path device)
          (fn copy-files [mountpoint]
            (let [src-files (-> (apply io/file
                                       (flatten [mountpoint (:activities-path device)]))
                                (.getCanonicalFile)
                                str)]
              (sh "rsync" "--archive" "--verbose" (str src-files "/") dest-dir)))))
      (println "Couldn't detect any Garmin device"))))
