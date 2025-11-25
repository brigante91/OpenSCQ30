import QtQuick 2.15
import QtQuick.Controls 2.15 as QtControls
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami
import org.kde.plasma.components 3.0 as PlasmaComponents

QtControls.ScrollView {
    id: configGeneral
    
    ColumnLayout {
        width: configGeneral.width
        
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            type: Kirigami.MessageType.Information
            text: "The widget automatically detects the OpenSCQ30 CLI. If it's not found, ensure it's compiled and in one of the standard locations."
            visible: true
        }
        
        QtControls.Label {
            Layout.fillWidth: true
            text: "Update Interval (seconds):"
        }
        
        QtControls.SpinBox {
            id: updateInterval
            from: 1
            to: 60
            value: plasmoid.configuration.updateInterval || 5
            onValueChanged: plasmoid.configuration.updateInterval = value
        }
        
        QtControls.Label {
            Layout.fillWidth: true
            Layout.topMargin: Kirigami.Units.largeSpacing
            text: "CLI Path (optional, leave empty for auto-detection):"
        }
        
        QtControls.TextField {
            id: cliPath
            Layout.fillWidth: true
            placeholderText: "Auto-detect"
            text: plasmoid.configuration.cliPath || ""
            onTextChanged: plasmoid.configuration.cliPath = text
        }
        
        QtControls.Button {
            Layout.fillWidth: true
            text: "Test Connection"
            onClicked: {
                // Trigger a test fetch
                dataSource.fetchData()
            }
        }
    }
}

