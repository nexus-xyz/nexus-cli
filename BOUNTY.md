# Nexus CLI Bounty Program: Bring the World Supercomputer to Every Device

*By Daniel Marin, Founder & CEO of Nexus*

At Nexus, we're building the **Layer-1 blockchain for the AI era** — a high-performance world supercomputer that can support the AI economy and scale to trillions of transactions per second. This isn't just a blockchain — but a decentralized, permissionless network capable of verifiable computation across the entire planet.

In this new computing paradigm, every node runs the **Nexus zkVM**, a zero-knowledge virtual machine that lets the network execute provable compute tasks — for AI, cryptography, simulation, and more — and stitch the results together like a single, global CPU.

We call this the **Layer-1 for the AI era**.

But to truly become a world supercomputer, Nexus must run everywhere. From datacenters to browsers, from phones to Teslas, from routers to smart watches. Any idle device capable of computation should be able to contribute, and in doing so, earn $NEX tokens in return.

That's where the **Nexus CLI** comes in.

---

## What is the Nexus CLI?

The [Nexus CLI](https://github.com/nexus-xyz/nexus-cli) is the lightweight interface that connects your device to the Nexus network and supplies verifiable compute. It's the "mining client" of Nexus — but instead of wasteful hashing, it performs real work.

The CLI runs the Nexus zkVM, accepts jobs from the network, and proves the results. The more compute you supply, the more you earn.

And here's the best part: the Nexus CLI is **cross-platform** and designed to run on *anything* with a CPU.

```
    +-----------------------+
    |                       |
    |     Your Device       |
    |                       |
    +----------+------------+
               |
               | Nexus CLI
               |
               v
    +----------+------------+
    |                       |
    |  Nexus Orchestrator   |<----> Nexus Network
    |                       |
    +-----------------------+
               ^
               |
    +----------+------------+
    |                       |
    |   Global Compute      |
    |                       |
    +-----------------------+
```

---

## Technical Architecture: How Nexus CLI Works

The Nexus CLI's architecture is specifically designed to enable cross-platform compatibility and efficient ZK proof generation. Here's a technical breakdown of how it works:

### Core Components

```
+------------------------------------------------------+
|                 Nexus CLI Architecture               |
+------------------------------------------------------+
|                                                      |
|  +------------+    +------------+    +------------+  |
|  |            |    |            |    |            |  |
|  | Node ID    |<-->| Prover     |<-->|Orchestrator|  |
|  | Manager    |    | Module     |    | Client     |  |
|  |            |    |            |    |            |  |
|  +------------+    +------------+    +------------+  |
|        ^                 ^                 ^         |
|        |                 |                 |         |
|        v                 v                 v         |
|  +------------+    +------------+    +------------+  |
|  |            |    |            |    |            |  |
|  | Analytics  |    | Nexus      |    | Network    |  |
|  | Module     |    | zkVM       |    | Interface  |  |
|  |            |    |            |    |            |  |
|  +------------+    +------------+    +------------+  |
|                                                      |
+------------------------------------------------------+
```

1. **Nexus zkVM**: The zero-knowledge virtual machine that executes programs and generates cryptographic proofs, implementing a RISC-V instruction set architecture.

2. **Prover Module**: Takes inputs (programs in RISC-V ELF format), executes them in the zkVM, and generates zero-knowledge proofs that verify computation without revealing the exact execution path.

3. **Orchestrator Client**: Communicates with the Nexus network to fetch proof tasks and submit completed proofs, handling authentication, task retrieval, and proof submission. This is the critical component that makes each device part of the global Nexus network.

4. **Node ID Manager**: Manages device identity on the network, allowing for authenticated participation and reward tracking.

5. **Analytics Module**: Monitors performance metrics to optimize proof generation across different hardware configurations.

### Network Interaction Flow

The core workflow for any Nexus CLI deployment follows this pattern:

```
+------------+    +------------+    +------------+    +------------+    +------------+
|            |    |            |    |            |    |            |    |            |
| 1. Node    |--->| 2. Task    |--->| 3. Proof   |--->| 4. Proof   |--->| 5. Reward  |
| Registration|    | Retrieval  |    | Generation |    | Submission |    | Calculation|
|            |    |            |    |            |    |            |    |            |
+------------+    +------------+    +------------+    +------------+    +------------+
      |                 ^                                   |                 |
      |                 |                                   v                 v
      |                 |                                   |                 |
      +-----------------+-----------------------------------+-----------------+
                                  Continuous Cycle
```

1. **Node Registration**: The device registers with the Nexus network using a unique node ID, which is used to track contributions and rewards.

2. **Task Retrieval**: The CLI regularly polls the Nexus Orchestrator to retrieve proof tasks that need to be processed.

3. **Proof Generation**: When a task is received, the CLI executes the requested program in the zkVM and generates a cryptographic proof of correct execution.

4. **Proof Submission**: The generated proof is submitted back to the Orchestrator, which verifies and records the contribution.

5. **Reward Calculation**: Based on the proofs submitted, the network calculates rewards in $NEX tokens for the contributing node.

This cycle continues as long as the CLI is running, allowing each device to contribute computational resources to the global network.

### Integration Workflow

To adapt the Nexus CLI to a new platform, implementers typically follow this pattern:

1. **Platform Assessment**: Analyze the target platform's CPU architecture, available memory, operating system constraints, and energy profile.

2. **Runtime Adaptation**: Implement platform-specific runtime adaptations for the zkVM to properly execute on constrained or specialized hardware.

3. **Background Service Configuration**: Develop appropriate patterns for running the CLI as a background service, often during device idle time or when connected to power.

4. **Power/Performance Optimization**: Implement platform-specific throttling and scheduling to balance proof generation with device primary functions and battery life.

5. **Installation/Distribution Method**: Create appropriate installation packages, containers, or embedded integrations depending on the platform's software ecosystem.

---

## Introducing the Nexus CLI Bounty Program

We're launching a public bounty program to **supercharge the reach of the Nexus CLI** — by getting the community to run it on the most diverse, exotic, and unexpected hardware platforms possible.

Some of the ideas are practical (smartphones, routers, servers). Others are wild (VR headsets, Teslas, 3D printers, even retro calculators). We want to stretch the imagination — and the network's edge.

```
                     +-------------------------+
                     |                         |
                     | Nexus Network           |
                     |                         |
                     +-----------+-------------+
                                 |
                                 |
           +-------------------------------------------+
           |                     |                     |
           v                     v                     v
  +----------------+    +----------------+    +----------------+
  |                |    |                |    |                |
  | Smart Devices  |    | Web Platforms  |    | Edge Computing |
  |                |    |                |    |                |
  +-------+--------+    +-------+--------+    +-------+--------+
          |                     |                     |
          v                     v                     v
  +---------------+     +---------------+     +---------------+
  | • Smart TVs   |     | • Browsers    |     | • Routers     |
  | • Smartphones |     | • Extensions  |     | • ATMs        |
  | • VR Headsets |     | • Discord Bots|     | • Kiosks      |
  | • Teslas/EVs  |     | • Minecraft   |     | • Billboards  |
  | • Appliances  |     |   Servers     |     | • 3D Printers |
  +---------------+     +---------------+     +---------------+
```

All bounties must result in:
- A working implementation of the Nexus CLI on the target device/platform
- Thorough documentation, posted as a PR to the [Nexus CLI GitHub repo](https://github.com/nexus-xyz/nexus-cli)
- Any necessary packaging or instructions for reproduction

---

## Bounty List

Here's a curated list of open bounties, sorted by category. Suggested rewards are paid in **testnet NEX** and will be honored for accepted contributions with sufficient documentation:

| Category              | Idea                                                                 | Difficulty | Reward    |
|-----------------------|----------------------------------------------------------------------|------------|-----------|
| Smart TVs             | Run Nexus CLI on Android TV / webOS in idle mode                     | Medium     | $1,000    |
| Wi-Fi Routers         | Integrate Nexus CLI into OpenWRT/DD-WRT routers                      | High       | $3,000    |
| Smart Appliances      | Nexus CLI on fridges, microwaves, etc.                               | High       | $2,500    |
| eReaders              | Background mining on Kindle or Kobo                                  | Medium     | $1,500    |
| Smartphones           | Background Android app for Nexus CLI                                 | Medium     | $2,000    |
| Smart Watches         | Idle-time compute on Wear OS or watchOS                              | High       | $2,500    |
| VR Headsets           | Idle mining on Meta Quest or Vision Pro                              | Medium     | $1,500    |
| Teslas / EVs          | Nexus CLI on Tesla OS while charging                                 | High       | $3,000    |
| Drones                | Mining while DJI drones are docked                                   | High       | $2,500    |
| Autonomous Tractors   | Nexus CLI on ROS-based farming machinery                             | High       | $3,500    |
| Gaming Consoles       | Jailbroken Xbox/PlayStation idle mining                              | Extreme    | $5,000    |
| Set-Top Boxes         | Nexus CLI on Roku / Apple TV                                         | Medium     | $1,500    |
| POS / ATMs            | Embedded Linux Nexus CLI for payment systems                         | High       | $3,000    |
| Airport Terminals     | Idle kiosk compute                                                   | Medium     | $2,000    |
| Smart Billboards      | Mining via smart signage compute units                               | High       | $3,000    |
| Calculators           | Nexus CLI on TI-89 or similar (for fun!)                             | Extreme    | $2,000    |
| Commodore 64          | Retro CLI in C64 BASIC/assembly                                      | Extreme    | $1,000    |
| Smart Mirrors         | Compute during idle display states                                   | Medium     | $1,500    |
| 3D Printers           | Idle-time Nexus CLI with OctoPrint                                   | Medium     | $1,500    |
| Dorm Pi Clusters      | Nexus CLI on student-run Pi clusters                                 | Low        | $750      |
| WebAssembly Browsers  | Fully in-browser Nexus miner                                         | High       | $3,500    |
| Chrome Extensions     | Background Nexus CLI as a browser extension                          | Medium     | $2,000    |
| Discord Bots          | CLI monitoring/reward gamification bots                              | Low        | $750      |
| Minecraft Plugin      | Nexus mining via Minecraft server plugins                            | Medium     | $1,500    |
| Auto-Deploy on LAN    | Script to install Nexus CLI on LAN devices automatically             | High       | $2,500    |
| Universal Packaging   | Nexus CLI packages (Snap, Docker, Flatpak, etc.)                     | Medium     | $1,000    |
| Kubernetes Sidecar    | Nexus CLI as a sidecar container                                     | High       | $3,000    |

---

## How to Submit

1. **Fork** the [Nexus CLI repo](https://github.com/nexus-xyz/nexus-cli)
2. Create your implementation and include:
   - Documentation (step-by-step setup, dependencies, notes)
   - Code, if needed (scripts, wrappers, packaging)
3. Submit a **pull request** with the bounty name in the title
4. Wait for review and confirmation
5. If accepted, you'll be rewarded in **testnet NEX**, and featured on our contributors page

---

## Why It Matters

Nexus isn't just a blockchain. It's a compute protocol for the next era — AI, cryptography, simulation — built on **verifiable, distributed, unstoppable compute**.

To get there, we need a **planetary substrate of hardware**, where anyone, anywhere can contribute compute — and know it was used meaningfully.

Whether it's a smart fridge or a space station, Nexus should run there.

This is your invitation to help build that reality.

Let's push the boundaries of what a blockchain node can be.

— **Daniel Marin**  
Founder & CEO, [Nexus](https://nexus.xyz)

---

## Appendix: Integration Example

Below is a simplified example of how to integrate the Nexus CLI into a new platform. This demonstrates the core pattern you'll typically follow when adapting the CLI to run on different devices.

### Proof Generation Workflow

The following diagram illustrates the complete workflow of proof generation and submission when integrating the Nexus CLI:

```
┌─────────────────────────────────┐         ┌─────────────────────────────────┐
│                                 │         │                                 │
│       Nexus Orchestrator        │         │        Target Device            │
│                                 │         │                                 │
└───────────────┬─────────────────┘         └───────────────┬─────────────────┘
                │                                           │
                │   1. Request task                         │
                │  <──────────────────────────────────────  │
                │                                           │
                │   2. Send proof task + program            │
                │  ─────────────────────────────────────►  │
                │                                           │
                │                                           │   3. Execute in zkVM
                │                                           │   ┌─────────────────┐
                │                                           │   │                 │
                │                                           ├──►│ Process task    │
                │                                           │   │                 │
                │                                           │   └────────┬────────┘
                │                                           │            │
                │                                           │            │
                │                                           │            ▼
                │                                           │   ┌─────────────────┐
                │                                           │   │                 │
                │                                           │   │ Generate proof  │
                │                                           │   │                 │
                │                                           │   └────────┬────────┘
                │                                           │            │
                │                                           │◄───────────┘
                │   4. Submit proof                         │
                │  <──────────────────────────────────────  │
                │                                           │
                │   5. Verify & acknowledge                 │
                │  ─────────────────────────────────────►  │
                │                                           │
                │   6. Record contribution                  │
                │   ┌─────────────────┐                     │
                │   │                 │                     │
                ├──►│ Update rewards  │                     │
                │   │                 │                     │
                │   └─────────────────┘                     │
                │                                           │
┌───────────────┴─────────────────┐         ┌───────────────┴─────────────────┐
│                                 │         │                                 │
│       Nexus Orchestrator        │         │        Target Device            │
│                                 │         │                                 │
└─────────────────────────────────┘         └─────────────────────────────────┘
```

### 1. Platform-Specific Wrapper

This is a sample wrapper script for integrating with a Linux-based embedded device that needs to run the prover during idle periods:

```rust
// platform_integration.rs
use std::time::{Duration, Instant};
use std::thread;
use std::process::Command;
use std::env;

fn is_device_idle() -> bool {
    // Platform-specific: Check CPU usage, user activity, etc.
    // ...
    return cpu_usage < 15.0;
}

fn is_on_external_power() -> bool {
    // Platform-specific implementation
    // ...
    return true;
}

fn main() {
    let check_interval = Duration::from_secs(60);
    let environment = "beta";
    
    // Get the node ID from environment variable or configuration
    let node_id = match env::var("NEXUS_NODE_ID") {
        Ok(id) => id,
        Err(_) => "anonymous".to_string()
    };
    
    println!("Starting Nexus CLI with node ID: {}", node_id);
    
    loop {
        if is_device_idle() && is_on_external_power() {
            // Start the Nexus CLI which will connect to the orchestrator
            let mut cmd = Command::new("nexus-cli");
            cmd.arg("start").arg("--env").arg(environment);
            
            if node_id != "anonymous" {
                cmd.env("NEXUS_NODE_ID", &node_id);
            }
            
            let prover = cmd.spawn().expect("Failed to start Nexus CLI");
            
            // Monitor system and let the prover run until conditions change
            // ...
            
            // Terminate the prover when conditions change
            prover.kill().expect("Failed to terminate Nexus CLI");
        }
        
        thread::sleep(check_interval);
    }
}
```

### 2. Custom Proof Task for Platform Constraints

This example shows how to fetch tasks from the Nexus orchestrator, generate proofs, and submit them back:

```rust
// platform_prover.rs
use nexus_sdk::{stwo::seq::Stwo, Local, Prover, Viewable};
use sha3::{Digest, Keccak256};
use std::path::Path;

pub async fn run_platform_prover(
    node_id: &str, 
    resource_level: u32,
    orchestrator_client: &OrchestratorClient
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Fetch a task from the orchestrator
    println!("Fetching a task from Nexus Orchestrator...");
    let proof_task = match orchestrator_client.get_proof_task(node_id).await {
        Ok(task) => task,
        Err(e) => return Err(e)
    };

    // 2. Extract inputs from the task
    let public_input = proof_task.public_inputs.first().cloned().unwrap_or_default() as u32;
    
    // 3. Select appropriate program based on device capabilities
    let program_file = match resource_level {
        0 => "tiny_program.elf",  // Very constrained devices
        // ... other options
        _ => "full_program.elf",  // Server class
    };
    
    // 4. Load the program and create prover
    let elf_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(program_file);
    
    let prover = Stwo::<Local>::new_from_file(&elf_file_path)?;
    
    // 5. Generate the zero-knowledge proof
    let (view, proof) = prover.prove_with_input::<(), u32>(&(), &public_input)?;
    
    // 6. Verify and prepare for submission
    // ... (verification code)
    
    // 7. Serialize the proof
    let proof_bytes = postcard::to_allocvec(&proof)?;
    let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));
    
    // 8. Submit the proof back to the orchestrator
    println!("Submitting ZK proof to Nexus Orchestrator...");
    match orchestrator_client.submit_proof(&proof_task.task_id, &proof_hash, proof_bytes).await {
        Ok(_) => {
            println!("✅ ZK proof successfully submitted to orchestrator");
            Ok(())
        },
        Err(e) => Err(e)
    }
}

// Implementation of a task scheduler for resource-constrained devices
pub async fn platform_task_scheduler(node_id: &str, environment: &config::Environment) 
-> Result<(), Box<dyn std::error::Error>> {
    let orchestrator_client = OrchestratorClient::new(environment.clone());
    
    // Continuously fetch and process tasks
    loop {
        if can_run_proofs() {
            match run_platform_prover(node_id, determine_device_capabilities(), &orchestrator_client).await {
                Ok(_) => println!("Proof task completed successfully"),
                Err(e) => println!("Error in proof task: {}", e),
            }
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
```

### 3. Platform-Specific Installation

This demonstrates how you might package the Nexus CLI for an Android application:

```java
// Android service example
public class NexusProverService extends Service {
    private boolean isRunning = false;
    private PowerManager.WakeLock wakeLock;
    private Thread proverThread;
    
    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        if (!isRunning) {
            // Acquire wake lock to keep CPU running
            PowerManager powerManager = (PowerManager) getSystemService(POWER_SERVICE);
            wakeLock = powerManager.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, 
                                            "NexusProver::ProverWakeLock");
            wakeLock.acquire();
            
            // Get the node ID from preferences
            SharedPreferences prefs = getSharedPreferences("NexusPrefs", MODE_PRIVATE);
            String nodeId = prefs.getString("node_id", null);
            
            // Start service in foreground with notification
            startForeground(NOTIFICATION_ID, createNotification("Nexus Prover is running"));
            
            // Start prover in separate thread
            proverThread = new Thread(() -> {
                try {
                    Log.i(TAG, "Connecting to Nexus Orchestrator...");
                    
                    // Call native code that handles the orchestrator interaction
                    boolean success = startNativeProver(nodeId, "beta");
                    
                    if (!success) {
                        Log.e(TAG, "Failed to start native prover");
                    }
                } catch (Exception e) {
                    Log.e(TAG, "Error in prover thread", e);
                }
            });
            proverThread.start();
            
            isRunning = true;
        }
        
        return START_STICKY;
    }
    
    // JNI method to call Rust prover - this will connect to the orchestrator
    private native boolean startNativeProver(String nodeId, String environment);
    
    // Load the native library
    static {
        System.loadLibrary("nexus_prover");
    }
    
    // ... cleanup code in onDestroy()
}
```

These examples illustrate the flexible architecture of the Nexus CLI and how it can be adapted to run on various platforms while maintaining the critical orchestrator-based workflow that connects each device to the global Nexus network.

---

**Track progress or contribute here:**  
[https://github.com/nexus-xyz/nexus-cli](https://github.com/nexus-xyz/nexus-cli)
