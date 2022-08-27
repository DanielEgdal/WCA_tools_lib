export function log(string) {
    console.log(string);
}

export function add_group(group) {
    let table = document.getElementById("table");
    let row = table.rows[0];
    let cell = row.cells[3 * group - 2];
    while (!cell) {
        row.insertCell();
        cell = row.cells[3 * group - 2];
    }
    cell.innerHTML = "Group: " + group;
}

export function add_name(str, group, index, groups) {
    let table = document.getElementById("table");
    let row = table.rows[index + 1];
    if (row == null) {
        row = table.insertRow();
        for (let i = 0; i < 3 * groups; i++) {
            row.insertCell();
        }
    }
    row.cells[3 * group + 1].innerHTML = str;
    if (group > 0) row.cells[3 * group].innerHTML = "<button id=\"g" + group + "i" + index + "l\">\<</button>";
    if (group < groups - 1) row.cells[3 * group + 2].innerHTML = "<button id=\"g" + group + "i" + index + "r\">\></button>";
}

export function onclick(id, f) {
    let button = document.getElementById(id);
    button.onclick = f;
}

export function move_td(oGroup, oIndex, dGroup, dIndex, groups) {
    let table = document.getElementById("table");
    let cell = table.rows[oIndex + 1].cells[oGroup];
    let newCell = table.rows[dIndex + 1].cells[dGroup];
    newCell.innerHTML = cell.innerHTML;
    cell.innerHTML = "";
    if (dGroup == 0 || dGroup == groups - 1) {
        newCell.innerHTML = "";
    }
    else if (oGroup == 0) {
        newCell.innerHTML = "<button id=\"g" + Math.floor(oGroup / 3) + "i" + oIndex + "l\">\<</button>";
    }
    else if (oGroup == groups - 1) {
        newCell.innerHTML = "<button id=\"g" + Math.floor(oGroup / 3) + "i" + oIndex + "r\">\></button>";
    }
}

export function group_len(group, groups) {
    let table = document.getElementById("table");
    for (let i = 0; true; i++) {
        let row = table.rows[i + 1];
        if (!row) {
            let row = table.insertRow();
            for (let j = 0; j < groups; j++) {
                row.insertCell();
            }
            return i;
        }
        let cell = row.cells[group];
        if (cell.innerHTML == "") {
            return i;
        }
    }
}

export function update_id(id, newId) {
    document.getElementById(id).id = newId;
}

export function href(id, str) {
    let a = document.getElementById(id);
    a.href = "pdf/" + a.query + "&groups=" + str;
}