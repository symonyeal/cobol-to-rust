       identification division.
       program-id. formatter-driver.
       data division.
       working-storage section.
       01 n binary-long.
       01 raw-number pic 99v99.
       01 formatted-number pic x(5).
       procedure division.
           perform varying n from 0 by 1 until n = 10000
               compute raw-number = n / 100
               call 'format-number' using raw-number formatted-number
               display formatted-number with no advancing
           end-perform
           perform varying n from 9999 by -1 until n < 0
               compute raw-number = n / 100
               call 'format-number' using raw-number formatted-number
               display formatted-number with no advancing
           end-perform
           stop run.
