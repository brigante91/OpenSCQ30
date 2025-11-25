import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.plasma.core 2.1 as PlasmaCore
import org.kde.kirigami 2.20 as Kirigami

PlasmoidItem {
    id: root
    
    property string deviceName: "Not Connected"
    property int batteryLevel: 0
    property bool isCharging: false
    property bool hasLeftRight: false
    property int batteryLeft: 0
    property int batteryRight: 0
    property bool isChargingLeft: false
    property bool isChargingRight: false
    property int caseBattery: -1
    property string connectionStatus: "disconnected"
    
    Plasmoid.title: "OpenSCQ30"
    Plasmoid.icon: "audio-headphones"
    Plasmoid.toolTipSubText: {
        if (connectionStatus === "connected") {
            if (hasLeftRight) {
                return `Left: ${batteryLeft}% | Right: ${batteryRight}%${caseBattery >= 0 ? ` | Case: ${caseBattery}%` : ""}`
            } else {
                return `Battery: ${batteryLevel}%${isCharging ? " (Charging)" : ""}`
            }
        } else {
            return "Not connected to any device"
        }
    }
    
    Plasmoid.compactRepresentation: CompactRepresentation {}
    Plasmoid.fullRepresentation: FullRepresentation {}
    
    property int updateInterval: plasmoid.configuration.updateInterval || 5
    
    Timer {
        id: updateTimer
        interval: updateInterval * 1000 // Convert seconds to milliseconds
        running: connectionStatus === "connected"
        repeat: true
        onTriggered: {
            dataSource.fetchData()
        }
    }
    
    PlasmaCore.DataSource {
        id: dataSource
        engine: "executable"
        connectedSources: []
        
        function fetchData() {
            // Get the script path relative to the widget location
            var scriptPath = plasmoid.file("", "backend/openscq30-widget-backend.sh")
            // Set custom CLI path if configured
            var env = {}
            if (plasmoid.configuration.cliPath) {
                env["OPENSCQ30_CLI_PATH"] = plasmoid.configuration.cliPath
            }
            connectedSources = [scriptPath]
        }
        
        onNewData: function(sourceName, data) {
            if (data.exitCode === 0) {
                try {
                    var json = JSON.parse(data.stdout)
                    deviceName = json.device_name || "Unknown Device"
                    connectionStatus = json.connection_status || "disconnected"
                    
                    if (connectionStatus === "connected") {
                        if (json.battery_type === "dual") {
                            hasLeftRight = true
                            batteryLeft = json.battery_left || 0
                            batteryRight = json.battery_right || 0
                            isChargingLeft = json.is_charging_left || false
                            isChargingRight = json.is_charging_right || false
                        } else {
                            hasLeftRight = false
                            batteryLevel = json.battery_level || 0
                            isCharging = json.is_charging || false
                        }
                        caseBattery = json.case_battery !== undefined ? json.case_battery : -1
                    } else {
                        hasLeftRight = false
                        batteryLevel = 0
                        isCharging = false
                        caseBattery = -1
                    }
                } catch (e) {
                    console.error("Error parsing JSON:", e)
                }
            } else {
                connectionStatus = "disconnected"
            }
            disconnectSource(sourceName)
        }
        
        Component.onCompleted: {
            fetchData()
        }
    }
}

