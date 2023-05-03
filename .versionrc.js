const re = /^version = "(\d\.\d\.\d)"$/m;

function readVersion(contents) {
	const matches = contents.match(re);
	return matches[1];
}

function writeVersion(contents, version) {
	return contents.replace(re, `version = "${version}"`);
}

const tracker = {
	filename: "Cargo.toml",
	updater: {
		readVersion,
		writeVersion
	}
};

module.exports = {
	// read version
	packageFiles: [tracker],
	// write version
	bumpFiles: [tracker]
};