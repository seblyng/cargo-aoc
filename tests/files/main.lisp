(with-open-file (in (second sb-ext:*posix-argv*))
  (format t "~a~%" (read-line in nil))
  (format t "~a~%" (read-line in nil)))
