# FTD Access Control Rule Optimizer

## Background

All Cisco FW platforms have limits on number of ACL entries supported. Though this number is pretty high sometimes if access policies gets out of control the number can grow rapidly. Here is example of supported number of rules per platform in 2020 

![sizing table](https://community.cisco.com/t5/image/serverpage/image-id/73366iEAEB138EA42D44C4/image-size/large?v=v2&px=999)
![sizing table](https://community.cisco.com/t5/image/serverpage/image-id/73367iD1CD3E25A3ECE12C/image-size/large?v=v2&px=999)

In 2025 those numbers are higher, but still not infinity. This tool intended to analyze policies configured on FTD and recommend actions to reduce number of ACE by cleaning up shadow items or merge items together.

## Calculation # of ACE

1. Use "show access-list element-count"
2. Use "show access-list"

Underneath math is pretty simple. Let's assume in a single rule we have
* Two source subnets
* Three destination subnets
* Four source TCP non-consecutive ports
* Five destomatopm TCP non-consecutive ports

The total size will be `2 * 3 * 4 * 5 = 120` ACEs

Then sum ACE across all rules and this will be a final number. 

## How to stay inside limit FPR9300 SM-56 (6 Mil ACEs)

What it would take to overflow 5% (300,000 ACE) of the biggest platform for big enterprise.

If enterprise has a single app that spread between two on-prem DC and three major Cloud Providers (Azure, AWS, GCP). It has multiple source and destination networks to controk as well as ports. 
* 2 x On-prem Data Centers, with each DC having 10 subnets
* 3 x CSPs, with 19 subnets in each CSP (for example during long migration)
* 20 source TCP/UDP ports
* 25 destination TCP/UDP ports
* no multicast

`(2 x 10 On-prem) x (3 x 10 CSP) x 20 sec ports x 25 dst ports = 300,000 ACE`

## Use case 

If you are getting closer to the limit - don't panic ! It is time to start thinking how to optimize your policies. Policies might not be beautifully arranged into groups anymore, but you;ll get an idea of "how to" tackle the problem. Crossing the limit does **NOT** mean that your policy stop working or box will crash. You probably can leave for another 5-10%. 

## How to use

1. Login to FTG CLI
2. Collect 'show access-control-config`
3. Run the scripy `frd-acl-optimizer analyze -f collected_output.txt`


