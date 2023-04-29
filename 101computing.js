(() => {
	document.getElementById("CPUWrapper").setAttribute("style", "width:auto;height:auto");
	document.getElementById("code").setAttribute("style", "width:auto;height:auto");
	document.querySelectorAll("textarea").forEach(e => e.setAttribute("style", "resize:both;"));

	let cpu = document.getElementById("CPU");
	cpu.setAttribute("style", "height:auto");

	let overdrive_mode = false;
	let overdrive = document.createElement("input");
	
	overdrive.id = "overdrive";
	overdrive.type = "checkbox";

	overdrive.onclick = () => {
		overdrive_mode = !overdrive_mode;

		if (overdrive_mode) {
			document.getElementById("clock").max = 2000000;
			let logFile = document.getElementById("logFile");
			logFile.checked = false;
			logOnOff = false;
			logFile.disabled = true;
			document.getElementById("logWrapper").setAttribute("style", "display:none");
		} else {
			document.getElementById("clock").max = 200;
			let logFile = document.getElementById("logFile");
			logFile.disabled = false;
			document.getElementById("logWrapper").setAttribute("style", "");
		}
	};

	if (document.getElementById("overdrive") == null) {
		let br = document.createElement("br");
		cpu.appendChild(br);
		cpu.appendChild(overdrive);
		cpu.append("Clock superspeed");
	}

	let ram_size = document.createElement("input");
	ram_size.id = "ram_size";
	ram_size.type = "text";

	ram_size.onchange = () => window.populate_ram(parseInt(ram_size.value));

	if (document.getElementById("ram_size") == null) {
		let br = document.createElement("br");
		cpu.appendChild(br);
		cpu.append("RAM Size: ");
		cpu.appendChild(ram_size);
	}
	
	window.populate_ram = size => {
		let table = document.querySelector("#RAM > tbody");

		table.innerHTML = "";

		for (let i = 0; i < size; i += 10) {
			let row = document.createElement("tr");
			let len = 0;

			for (let j = 0; j < size - i && j < 10; j++) {
				if (document.getElementById("cell_"+(i+j)) != null) {
					continue;
				}

				len += 1;

				let cell = document.createElement("td");

				cell.innerHTML = (i+j);

				let input = document.createElement("input");
				input.id = "cell_"+(i+j);
				input.type = "text";
				input.setAttribute("value", "000");

				cell.appendChild(input);

				row.appendChild(cell);
			}
			if (len > 0) {
				table.appendChild(row);
			}
		}
	}
})();