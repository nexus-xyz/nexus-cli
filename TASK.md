
---

```markdown
# TASK.md — Minimal CLI UI for “All Your Node” (Screen-record friendly)

**Goal:** Add one command to `nexus-cli` that renders the script verbatim with simple timing, an activity log, a minimal SYN packet cloud, and an ASCII “SYN” outro with the year.  
This is just for screen recording; voices will be added later in iMovie.

---

## 1) Script (verbatim) + Cast (hex tags)

```

0xDEAD: In A.D. 20,1,5, SYN was beginning.
0xCABB: What happen?
0xF1X3: Somebody set up us the cron.
0xD00D: We get signal.
0xCABB: What!
0xD00D: Main screen turn on.
0xCABB: It's you!!
0xACCC: How are you sysadmins!!
0xACCC: All your node are belong to us.
0xACCC: You are on the way to destruction.
0xCABB: What you say!!
0xACCC: You have no chance to survive — make your time.
0xACCC: Ha ha ha ha....
0xD00D: 0xCABB!!
0xCABB: Take off every 'SYNC'!!
0xCABB: You know what you doing.
0xCABB: Move 'SYNC'.
0xCABB: For great justice.

```

---

## 2) File layout

```

.
├─ scripts/syn/all-your-node.scenes.json
├─ scripts/syn/activity.log.json
├─ assets/ascii/outro-syn.txt
└─ src/commands/syn/recruit.ts

````

---

## 3) Data files

### `scripts/syn/all-your-node.scenes.json`
```json
[
  {"speaker":"0xDEAD","line":"In A.D. 20,1,5, SYN was beginning.","delayMs":1200},
  {"speaker":"0xCABB","line":"What happen?","delayMs":700},
  {"speaker":"0xF1X3","line":"Somebody set up us the cron.","delayMs":800},
  {"speaker":"0xD00D","line":"We get signal.","delayMs":600},
  {"speaker":"0xCABB","line":"What!","delayMs":400},
  {"speaker":"0xD00D","line":"Main screen turn on.","delayMs":700},
  {"speaker":"0xCABB","line":"It's you!!","delayMs":600},
  {"speaker":"0xACCC","line":"How are you sysadmins!!","delayMs":800},
  {"speaker":"0xACCC","line":"All your node are belong to us.","delayMs":900},
  {"speaker":"0xACCC","line":"You are on the way to destruction.","delayMs":900},
  {"speaker":"0xCABB","line":"What you say!!","delayMs":700},
  {"speaker":"0xACCC","line":"You have no chance to survive — make your time.","delayMs":900},
  {"speaker":"0xACCC","line":"Ha ha ha ha....","delayMs":900},
  {"speaker":"0xD00D","line":"0xCABB!!","delayMs":500},
  {"speaker":"0xCABB","line":"Take off every 'SYNC'!!","delayMs":900},
  {"speaker":"0xCABB","line":"You know what you doing.","delayMs":700},
  {"speaker":"0xCABB","line":"Move 'SYNC'.","delayMs":700},
  {"speaker":"0xCABB","line":"For great justice.","delayMs":1000}
]
````

### `scripts/syn/activity.log.json`

```json
[
  {"level":"INFO","msg":"Boot sequence: SYN system online."},
  {"level":"WARN","msg":"Unexpected cron entry detected: \"@reboot ./payload.sh\""},
  {"level":"ALERT","msg":"Inbound transmission SRC=0xACCC → DST=0xCABB [ACK]"},
  {"level":"CRIT","msg":"Node takeover claim detected."},
  {"level":"ERROR","msg":"Firewall rule update rejected."},
  {"level":"INFO","msg":"INITIATING SYN LAUNCH..."},
  {"level":"OK","msg":"FOR GREAT JUSTICE"}
]
```

---

## 4) ASCII outro

### `assets/ascii/outro-syn.txt`

```
██████╗          ██╗   ██╗      ███╗   ██╗
██╔══██╗         ╚██╗ ██╔╝      ████╗  ██║
██████╔╝          ╚████╔╝       ██╔██╗ ██║
██╔══██╗           ╚██╔╝        ██║╚██╗██║
██████╔╝            ██║         ██║ ╚████║
╚═════╝             ╚═╝         ╚═╝  ╚═══╝

                 A.D. 20,1,5
```

---

## 5) Command implementation

### `src/commands/syn/recruit.ts`

```ts
import fs from "node:fs";
import path from "node:path";

const C = {
  reset:"\x1b[0m", bold:"\x1b[1m", dim:"\x1b[2m",
  gray:"\x1b[90m", yellow:"\x1b[33m", green:"\x1b[32m",
  cyan:"\x1b[36m", magenta:"\x1b[35m", red:"\x1b[31m"
};

type Scene = { speaker:string; line:string; delayMs:number };
type Log = { level:string; msg:string };

const sleep = (ms:number)=>new Promise(r=>setTimeout(r,ms));
const now = ()=>new Date().toISOString().replace("T"," ").replace("Z","");

const speakerColor = (s:string)=>{
  if (s==="0xACCC") return C.magenta;
  if (s==="0xCABB") return C.yellow;
  if (s==="0xF1X3") return C.green;
  if (s==="0xD00D") return C.cyan;
  if (s==="0xDEAD") return C.gray;
  return C.reset;
};

const levelColor = (l:string)=>({
  INFO:C.cyan, WARN:C.yellow, ALERT:C.magenta, CRIT:C.red,
  ERROR:C.red, OK:C.green
}[l]||C.gray);

async function printActivity(logs:Log[]) {
  console.log(C.dim+"── Activity Log ───────────────────────────────"+C.reset);
  for (const e of logs) {
    console.log(`${C.gray}[${now()}]${C.reset} ${levelColor(e.level)}${e.level}${C.reset} ${e.msg}`);
    await sleep(200);
  }
  console.log(C.dim+"───────────────────────────────────────────────"+C.reset);
}

async function packetCloud(ms=1200) {
  const frames=["->SYN   ","  ->SYN","->SYN ->SYN"];
  const end=Date.now()+ms; let i=0;
  while(Date.now()<end){
    process.stdout.write("\x1b[2J\x1b[H");
    console.log(C.cyan+"SYN PACKETS LAUNCHING..."+C.reset+"\n");
    for(let r=0;r<6;r++) console.log(C.cyan+frames[(i+r)%frames.length]+C.reset);
    await sleep(80); i++;
  }
}

async function printAscii(file:string,color=C.green){
  const p=path.resolve(file);
  if(!fs.existsSync(p))return;
  for(const l of fs.readFileSync(p,"utf8").split("\n")){
    console.log(color+l+C.reset); await sleep(15);
  }
}

export async function runSynRecruit(){
  const scenes:Scene[]=JSON.parse(fs.readFileSync("scripts/syn/all-your-node.scenes.json","utf8"));
  const logs:Log[]=JSON.parse(fs.readFileSync("scripts/syn/activity.log.json","utf8"));

  process.stdout.write("\x1b[2J\x1b[H");
  console.log(C.gray+"BOOT> INITIALIZING SYN SYSTEM ..."+C.reset);
  await sleep(400);

  for(const s of scenes){
    console.log(`${C.bold}${speakerColor(s.speaker)}${s.speaker}${C.reset}: ${s.line}`);
    await sleep(s.delayMs);

    if(s.line.includes("Take off every 'SYNC'")){
      await printActivity(logs.slice(0,5));
      await packetCloud(1200);
      await printActivity(logs.slice(5));
    }
  }

  await printAscii("assets/ascii/outro-syn.txt",C.green);
  console.log(C.dim+"Broadcast complete. Packet dropped."+C.reset);
}
```

**Wire it up**

```ts
// src/index.ts
import { Command } from "commander";
import { runSynRecruit } from "./commands/syn/recruit";

const program = new Command();
program
  .command("syn")
  .description("SYN utilities")
  .command("recruit")
  .description("Play minimal recruitment UI")
  // @ts-ignore
  .action(runSynRecruit);

program.parseAsync(process.argv);
```

**Run**

```bash
npm run build
node dist/index.js syn recruit
# or nexus-cli syn recruit
```

---

## 6) Screen recording

* Use dark terminal theme, big readable font.
* macOS: `Shift+Cmd+5` → Record Selected Portion → select terminal window.
* Run once cleanly for capture.
* Add monotone voice in iMovie later.

---

```

---

```
