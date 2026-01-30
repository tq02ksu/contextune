package com.contextune.plugin.ui

import java.awt.datatransfer.DataFlavor
import java.awt.datatransfer.Transferable
import java.awt.datatransfer.UnsupportedFlavorException
import java.awt.dnd.*
import javax.swing.*
import kotlin.text.compareTo

/**
 * Drag and drop handler for playlist track reordering
 */
class PlaylistDragDropHandler(
    private val trackList: JList<TrackItem>,
    private val trackListModel: DefaultListModel<TrackItem>,
    private val onReorder: (fromIndex: Int, toIndex: Int) -> Unit
) : DropTargetListener, DragGestureListener, DragSourceListener {
    
    private var draggedIndex: Int = -1
    private val dragSource = DragSource()
    private val dropTarget = DropTarget(trackList, this)
    
    init {
        // Enable drag and drop
        trackList.dragEnabled = true
        trackList.dropMode = DropMode.INSERT
        
        // Set up drag source
        dragSource.createDefaultDragGestureRecognizer(
            trackList,
            DnDConstants.ACTION_MOVE,
            this
        )
    }
    
    // DragGestureListener implementation
    override fun dragGestureRecognized(dge: DragGestureEvent) {
        val selectedIndex = trackList.selectedIndex
        if (selectedIndex >= 0) {
            draggedIndex = selectedIndex
            val trackItem = trackListModel.getElementAt(selectedIndex)
            val transferable = TrackTransferable(trackItem, selectedIndex)
            
            try {
                dge.startDrag(DragSource.DefaultMoveDrop, transferable, this)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
    
    // DragSourceListener implementation
    override fun dragEnter(dsde: DragSourceDragEvent) {}
    override fun dragOver(dsde: DragSourceDragEvent) {}
    override fun dropActionChanged(dsde: DragSourceDragEvent) {}
    override fun dragExit(dse: DragSourceEvent) {}
    override fun dragDropEnd(dsde: DragSourceDropEvent) {
        draggedIndex = -1
    }
    
    // DropTargetListener implementation
    override fun dragEnter(dtde: DropTargetDragEvent) {
        if (dtde.isDataFlavorSupported(TrackTransferable.TRACK_FLAVOR)) {
            dtde.acceptDrag(DnDConstants.ACTION_MOVE)
        } else {
            dtde.rejectDrag()
        }
    }
    
    override fun dragOver(dtde: DropTargetDragEvent) {
        if (dtde.isDataFlavorSupported(TrackTransferable.TRACK_FLAVOR)) {
            dtde.acceptDrag(DnDConstants.ACTION_MOVE)
        } else {
            dtde.rejectDrag()
        }
    }
    
    override fun dropActionChanged(dtde: DropTargetDragEvent) {}
    
    override fun dragExit(dte: DropTargetEvent) {}
    
    override fun drop(dtde: DropTargetDropEvent) {
        try {
            if (dtde.isDataFlavorSupported(TrackTransferable.TRACK_FLAVOR)) {
                dtde.acceptDrop(DnDConstants.ACTION_MOVE)
                
                val transferable = dtde.transferable
                val trackData = transferable.getTransferData(TrackTransferable.TRACK_FLAVOR) as TrackTransferData
                
                // Calculate drop location
                val dropLocation = trackList.dropLocation
                val dropIndex = if (dropLocation != null) {
                    dropLocation.index
                } else {
                    trackListModel.size()
                }
                
                // Perform the reorder
                if (draggedIndex >= 0 && draggedIndex != dropIndex) {
                    val adjustedDropIndex = if (dropIndex > draggedIndex) dropIndex - 1 else dropIndex
                    
                    // Move the item in the model
                    val trackItem = trackListModel.elementAt(draggedIndex)
                    trackListModel.removeElementAt(draggedIndex)
                    val insertIndex = if (draggedIndex < adjustedDropIndex) adjustedDropIndex - 1 else adjustedDropIndex
                    trackListModel.insertElementAt(trackItem, insertIndex)
                    // Update selection
                    trackList.selectedIndex = adjustedDropIndex
                    
                    // Notify callback
                    onReorder(draggedIndex, adjustedDropIndex)
                }
                
                dtde.dropComplete(true)
            } else {
                dtde.rejectDrop()
            }
        } catch (e: Exception) {
            e.printStackTrace()
            dtde.rejectDrop()
        }
    }
}

/**
 * Transferable implementation for track items
 */
class TrackTransferable(
    private val trackItem: TrackItem,
    private val originalIndex: Int
) : Transferable {
    
    companion object {
        val TRACK_FLAVOR = DataFlavor(TrackTransferData::class.java, "Track Item")
    }
    
    override fun getTransferDataFlavors(): Array<DataFlavor> {
        return arrayOf(TRACK_FLAVOR)
    }
    
    override fun isDataFlavorSupported(flavor: DataFlavor): Boolean {
        return flavor == TRACK_FLAVOR
    }
    
    override fun getTransferData(flavor: DataFlavor): Any {
        if (flavor == TRACK_FLAVOR) {
            return TrackTransferData(trackItem, originalIndex)
        }
        throw UnsupportedFlavorException(flavor)
    }
}

/**
 * Data class for transferring track information
 */
data class TrackTransferData(
    val trackItem: TrackItem,
    val originalIndex: Int
)