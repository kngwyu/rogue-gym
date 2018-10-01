let fs = require('fs');
let path = require('path')
function is_dir(file_path) {
    return fs.existsSync(file_path) && fs.statSync(file_path).isDirectory();
}
function format_file(file_name) {
    let ext = path.extname(file_name);
    if (ext != '.json') {
        return false;
    }
    let file = fs.readFileSync(file_name, 'utf8');
    let json = JSON.parse(file);
    fs.writeFileSync(file_name, JSON.stringify(json, null, 4))
    return true;
}
function format_directory(dir_name) {
    let sum = 0;
    fs.readdirSync(dir_name).forEach(function(file_name) {
        let name = path.join(dir_name, file_name);
        if (is_dir(name)) {
            sum += format_directory(name);
        } else if (format_file(name)) {
            sum += 1;
        }
    });
    return sum;
}
let args = process.argv;
if (args.length < 3) {
    console.log('usage: node format-json.js <filename>/<dirname>')
    process.exit()
}
let name = args[2];
if (is_dir(name)) {
    let sum = format_directory(name);
    console.log('formatted ' + sum + ' files')
} else {
    if (format_file(name)) {
        console.log('formatted ' + name)
    } else {
        console.log('Error: ' + name + 'is not a json file')
    }
}
