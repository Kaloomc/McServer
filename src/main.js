const invoke = window.__TAURI__?.invoke ?? window.__TAURI__.core?.invoke;

const statusIntervals = new Map();

// Fonction pour mettre à jour le statut d'un serveur
async function updateServerStatus(dotElement, folderName) {
  const isRunning = await invoke("is_server_running", { folderName });
  if (isRunning) {
    dotElement.style.backgroundColor = "#00FF00";
    document.getElementById("StartButton-" + folderName).textContent = "Stop";
    document.getElementById("StartButton-" + folderName).removeEventListener("click", start_server);
    document.getElementById("StartButton-" + folderName).addEventListener("click", stop_server);
  } else {
    dotElement.style.backgroundColor = "#FF0000";
    document.getElementById("StartButton-" + folderName).textContent = "Start";
    document.getElementById("StartButton-" + folderName).removeEventListener("click", stop_server);
    document.getElementById("StartButton-" + folderName).addEventListener("click", start_server);
  }
}

async function Search_folder_server() {
  // nettoyer timers existants pour éviter doublons
  statusIntervals.forEach(id => clearInterval(id));
  statusIntervals.clear();

  const paths = await invoke("get_data_folder_list");
  
  const element = document.getElementById("contain");
  element.innerHTML = "";

  paths.forEach(async folder => {
    //create box
    const newDiv = document.createElement("div");
    newDiv.className = "box";
    newDiv.id = folder;
    element.append(newDiv); 
    
    //create headbar
    const newHeadBar = document.createElement("div");
    newHeadBar.className = "headBar";
    newDiv.append(newHeadBar);

    //create server title
    const newTitle = document.createElement("div");
    newTitle.className = "serverName";
    newTitle.textContent = folder;
    newHeadBar.append(newTitle);

    //create player box
    const newPlayersNumber = document.createElement("div");
    newPlayersNumber.className = "playersNumber";
    newHeadBar.append(newPlayersNumber);

    //create player title
    const newPlayersNumberText = document.createElement("div");
    newPlayersNumberText.className = "playersNumberText";
    newPlayersNumberText.textContent = "0/20";
    newPlayersNumber.append(newPlayersNumberText);

    //create version x.xx.x
    const newVersionText = document.createElement("div");
    newVersionText.className = "versionText";
    const v = await invoke("get_server_version", { folderName: folder })
    newVersionText.textContent = v;
    newHeadBar.append(newVersionText);
    
    //create Description server
    const newDescriptionText = document.createElement("div");
    newDescriptionText.className = "descriptionText";
    const desc = await invoke("get_description_server", { folderName: folder })
    newDescriptionText.textContent = desc;
    newDiv.append(newDescriptionText);
    
    //create Button
    const ButtonHolder = document.createElement("div");
    ButtonHolder.className = "ButtonHolder";
    newDiv.append(ButtonHolder);

    //Start
    const newStartButton = document.createElement("button");
    newStartButton.id = 'StartButton-' + folder;
    newStartButton.className = "buttonStart";
    newStartButton.textContent = "Start";
    newStartButton.addEventListener("click", start_server);
   
    ButtonHolder.append(newStartButton);

    //Edit
    const newEditButton = document.createElement("button");
    newEditButton.className = "buttonEdit";
    newEditButton.textContent = "Edit";
    ButtonHolder.append(newEditButton);

    //Folder
    const newFolderButton = document.createElement("button");
    newFolderButton.className = "buttonFolder";
    newFolderButton.textContent = "Folder";

    newFolderButton.addEventListener("click", () => {
      const id = newFolderButton.closest(".box").id;
      invoke("open_folder", { folderName: id });
    });

    ButtonHolder.append(newFolderButton);

    //create dotstatus
    const newDotStatus = document.createElement("div");
    newDotStatus.className = "dotStatus";
    newDotStatus.style.backgroundColor = "#FF0000"
    
    await updateServerStatus(newDotStatus, folder);
    const intervalId = setInterval(() => updateServerStatus(newDotStatus, folder), 2000);
    statusIntervals.set(folder, intervalId);
    
    newHeadBar.prepend(newDotStatus);
  });
}

async function start_server() {
  const id = this.closest(".box").id;
  await invoke("open_server", { folderName: id });
}

async function stop_server() {
  const id = this.closest(".box").id;
  await invoke("stop_server", { folderName: id });
}

// Open popup
function open_popup() {
  document.getElementById("PopUp").style.display = "flex";
}

// Close popup
function close_popup() {
  document.getElementById("PopUp").style.display = "none";
}

// Create new server from popup
async function create_server() {
  const serverName = document.getElementById("ServerName").value.trim();
  const description = document.getElementById("ServerDescription").value || "A Minecraft Server";
  const version = document.getElementById("VersionDropdown").value;
  const maxPlayers = document.getElementById("MaxPlayers").value;
  const difficulty = document.getElementById("Difficulty").value;
  const whitelist = document.getElementById("Whitelist").checked;
  const cracked = document.getElementById("Cracked").checked;
  const allowFlight = document.getElementById("AllowFlight").checked;
  const forceGamemode = document.getElementById("ForceGamemode").checked;
  const spawnProtection = document.getElementById("SpawnProtection").value;

  if (!serverName) {
    alert("Le nom du serveur est requis!");
    return;
  }

  if (!version) {
    alert("Veuillez sélectionner une version!");
    return;
  }

  // Create the server folder
  await invoke("create_new_data_folder", { folderName: serverName });

  // TODO: You'll need to add a Rust command to create server.properties with these settings
  // For now, just create the folder
  
  close_popup();
  await Search_folder_server();
}

// Load Paper versions into dropdown
async function loadPaperVersions() {
  try {
    const versions = await invoke("get_paper_versions");
    const listEl = document.getElementById("VersionDropdown");
    
    listEl.innerHTML = "";
    
    // Add versions in reverse order (newest first)
    versions.reverse().forEach(version => {
      const option = document.createElement("option");
      option.value = version;
      option.textContent = `Paper ${version}`;
      listEl.appendChild(option);
    });
  } catch (error) {
    console.error("Failed to load versions:", error);
  }
}

// Event listeners
document.getElementById("add").addEventListener("click", open_popup);
document.getElementById("CreateServerBtn").addEventListener("click", create_server);
document.getElementById("CancelBtn").addEventListener("click", close_popup);

// Close popup when clicking outside
document.getElementById("PopUp").addEventListener("click", (e) => {
  if (e.target.id === "PopUp") {
    close_popup();
  }
});

// Initialize
window.addEventListener("DOMContentLoaded", () => {
  loadPaperVersions();
  Search_folder_server();
});