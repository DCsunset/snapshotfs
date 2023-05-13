const re = /name = "snapshotfs"\nversion = "(\d\.\d\.\d)"/;

function readVersion(contents) {
	const matches = contents.match(re);
	return matches[1];
}

function writeVersion(contents, version) {
	return contents.replace(re, `name = "snapshotfs"\nversion = "${version}"`);
}

const updater = { readVersion, writeVersion };

const trackers = [
	{
		filename: "Cargo.toml",
		updater
	},
	{
		filename: "Cargo.lock",
		updater
	}
];

module.exports = {
	// read version
	packageFiles: trackers,
	// write version
	bumpFiles: trackers
};