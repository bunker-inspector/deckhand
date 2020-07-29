system resource tracker

Build and run with
```
./deckhand \
  --logger print
  #The follow are all defaults
  --baseline-duration 10\
  --polling-interval  5\
  --standard-deviation-threshold 3

# Output ...
# Normal(ResourceUsage { mem: 0.6414914, cpu: 0.038803842, swap: 0.5135905, temps: {"Battery": 30.1875, "CPU": 45.625} })
# Normal(ResourceUsage { mem: 0.63547444, cpu: 0.090810694, swap: 0.5135905, temps: {"CPU": 46.0625, "Battery": 30.1875} })
# Normal(ResourceUsage { mem: 0.6354196, cpu: 0.03631567, swap: 0.5135905, temps: {"CPU": 44.125, "Battery": 30.1875} })
```
