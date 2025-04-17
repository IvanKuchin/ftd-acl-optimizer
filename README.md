# FTD Access Control Rule Optimizer

## Background

All Cisco FW platforms have limits on number of ACL entries supported. Though this number is pretty high sometimes if access policies gets out of control the number can grow rapidly. Here is example of supported number of rules per platform in 2020 

![sizing table](https://community.cisco.com/t5/image/serverpage/image-id/73366iEAEB138EA42D44C4/image-size/large?v=v2&px=999)
![sizing table](https://community.cisco.com/t5/image/serverpage/image-id/73367iD1CD3E25A3ECE12C/image-size/large?v=v2&px=999)

In 2025 those numbers are higher, but still not infinity. This tool intended to analyze policies configured on FTD and recommend actions to reduce number of ACE by cleaning up shadow items or merge items together.

## Use case 

If you are getting closer to the limit - don't panic ! It is time to start thinking how to optimize your policies. Policies might not be beautifully arranged into groups anymore, but you;ll get an idea of "how to" tackle the problem. Crossing the limit does **NOT** mean that your policy stop working or box will crash. You probably have runway for another 5-10%. 

## How to use

1. Login to FTD CLI
3. Collect 'show access-control-config`
4. Run the script `ftd-acl-optimizer analyze -f collected_output.txt`

## Cisco solution

CDO (Cisco Defense Orchestrator) can analyze policy and produce report. Integrate FMC with CDO then navigate to [Policy insight](https://docs.defenseorchestrator.com/?cid=manage_ftd#!t-policy-insights-.html)

If you don't have access to either of these products - keep reading.

## Calculation # of ACE

1. Use `show access-list element-count`
2. Use `show access-list`

Underneath math is pretty simple. Let's assume in a single rule we have
* Two source subnets
* Three destination subnets
* Four source TCP non-consecutive ports
* Five destomatopm TCP non-consecutive ports

The total size will be `2 * 3 * 4 * 5 = 120` ACEs

Then sum ACE across all rules and this will be a final number. 

## How to stay inside limit FPR9300 SM-56 (6 Mil ACEs)

What it would take to overflow 5% (300,000 ACE) of the biggest platform (SM-56) for large enterprise.

If enterprise has a single app that spread between two on-prem highly available Data Centers and three major Cloud Providers (Azure, AWS, GCP). It has multiple source and destination networks to controk as well as ports. 
* 2 x On-prem Data Centers, with each DC having 10 subnets
* 3 x CSPs, with 10 subnets in each CSP (for example during long migration)
* 20 source TCP/UDP ports
* 25 destination TCP/UDP ports
* no multicast

`(2 x 10 On-prem) x (3 x 10 CSP) x 20 src ports x 25 dst ports = 300,000 ACE`

## Optimizations

### Built-in optimizations to FTD

1. IP range optimized to subnets inside network objects and groups.  
   For example:  
   `192.168.0.0 - 192.168.0.5`  
   Will be optimized to two subnets: `192.168.0.0/30` and `192.168.0.4/31`  
2. Adjacent/overlap/shadow/etc ... layer 4 ports (TCP and UDP)
   For example:  
   `SSH TCP 22`  and `FTP TCP 21`  
   Will be optimized to a contiguous range:
   `TCP 21-22`

### App optimization

Due to port optimizations are done automatically, app only optimizes Adjacent/Overlap/Shadow subnets inside rules (not across the whole access policy).  
IMPORTANT: Optimizations across rules are outside of the project scope.

Example: RULE my_app
- Source subnets
  - `192.168.168.0/25`
  - `192.168.168.128/25`
- Destination subnets
  - `10.11.12.0/24`
  - `10.11.13.0/24`
- Source ports
  - `ephemeral`
  - `FTP`
- Destination ports
  - `HTTPS`
  - `FTP`
  
Source subnets will be optimized to a single: 192.168.168.0/24  
Destination subnets will be optimized to a single: 10.11.12.0/23  
Destination ports will be optimized by FTD-code
   
Number of ACE before optimization: 
2 (src subnets) * 2 (dst subnets) * 2 (src ports) * 2 (dest ports) = 16

Number of ACE after optimization: 
1 (src subnets) * 1 (dst subnets) * 2 (src ports) * 2 (dest ports) = 4

Optimization factor: 16 / 4 = 4 

### Types of optimization

* Shadow - Example: `192.168.168.0/24` shadows by `192.168.0.0/16` (factor 2)
* Overlap - Example: IP range `192.168.168.0-254` overlaps with `192.168.168.1-255` should be optimized to a single subnet `192.168.168.0/24` (factor 64)
* Adjacency - Example: `192.168.168.0/25` and `192.168.168.128/25` optimizes to `192.168.168.0/24` (factor 2)


